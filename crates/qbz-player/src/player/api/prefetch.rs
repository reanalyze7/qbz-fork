use super::*;

impl Player {
    pub async fn prefetch_into_cache(
        &self,
        client: &QobuzClient,
        track_id: u64,
        quality: Quality,
    ) -> Result<(), String> {
        // Honor streaming_only — never warm the cache when the user has
        // opted out of caching.
        let skip_cache = self
            .audio_settings
            .lock()
            .map(|s| s.streaming_only)
            .unwrap_or(false);
        if skip_cache {
            log::debug!("[PREFETCH] Skipped track {track_id} — streaming_only mode active");
            return Ok(());
        }

        // Already cached, or another prefetch for this id is already
        // running — nothing to do.
        if self.audio_cache.contains(track_id) {
            log::debug!("[PREFETCH] Track {track_id} already cached");
            return Ok(());
        }
        if self.audio_cache.is_fetching(track_id) {
            log::debug!("[PREFETCH] Track {track_id} already being fetched");
            return Ok(());
        }
        // Back off a track whose last prefetch failed recently (e.g. the account
        // is being 403'd post-outage) instead of re-hammering it on every queue
        // tick — that no-backoff loop is what escalated into an edge/IP block in
        // issue #637. The client-side 403 breaker is the primary guard; this
        // just keeps us from even spinning up the task/log spam.
        const PREFETCH_FAIL_COOLDOWN: Duration = Duration::from_secs(20);
        if self
            .audio_cache
            .recently_failed(track_id, PREFETCH_FAIL_COOLDOWN)
        {
            log::debug!("[PREFETCH] Track {track_id} recently failed — backing off");
            return Ok(());
        }

        self.audio_cache.mark_fetching(track_id);
        log::info!("[PREFETCH] Prefetching track {track_id} at {quality:?}");

        // Try CMAF full download first (Akamai CDN), legacy full download
        // as fallback (nginx CDN).
        let result = match qbz_qobuz::cmaf::download_full(client, track_id, quality).await {
            Ok(data) => Ok(data),
            Err(e) => {
                log::warn!(
                    "[PREFETCH] CMAF failed for track {track_id}: {e}, trying legacy"
                );
                match client.get_stream_url_with_fallback(track_id, quality).await {
                    Ok(stream_url) => self.download_audio(&stream_url.url).await,
                    Err(e) => Err(format!("Failed to get stream URL: {e}")),
                }
            }
        };

        match result {
            Ok(data) => {
                // Brief delay before the cache write to avoid racing the
                // audio thread, matching the Tauri prefetch path.
                tokio::time::sleep(Duration::from_millis(50)).await;
                let len = data.len();
                self.audio_cache.insert(track_id, data);
                self.audio_cache.unmark_fetching(track_id);
                self.audio_cache.clear_failed(track_id);
                log::info!("[PREFETCH] Complete for track {track_id} ({len} bytes)");
                Ok(())
            }
            Err(e) => {
                self.audio_cache.unmark_fetching(track_id);
                self.audio_cache.mark_failed(track_id);
                log::warn!("[PREFETCH] Failed for track {track_id}: {e}");
                Err(e)
            }
        }
    }

    /// True if `track_id` is present in the L1/L2 playback cache. Used by
    /// the gapless controller to decide whether a track can be queued for
    /// a seamless handoff.
    pub fn is_track_cached(&self, track_id: u64) -> bool {
        self.audio_cache.contains(track_id)
    }

    /// Drop every cached audio byte (L1 memory + L2 disk). Called when the
    /// streaming-quality preference changes so the new tier takes effect on the
    /// next play/cast instead of on the next cache miss — the cache is keyed by
    /// track id alone and carries no quality dimension.
    pub fn clear_audio_cache(&self) {
        self.audio_cache.clear();
    }
}
