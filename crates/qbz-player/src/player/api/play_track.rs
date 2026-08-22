use super::*;

mod play_track_cmaf;
mod play_track_legacy;

impl Player {
    /// Play a track by ID.
    ///
    /// First attempts the CMAF streaming pipeline (Akamai CDN, encrypted
    /// segments): only the init segment is fetched synchronously to derive
    /// stream parameters, playback starts immediately, and audio segments are
    /// fetched + decrypted + pushed to the streaming buffer in a background
    /// task. If the CMAF setup fails for any reason, falls back to the legacy
    /// `/track/getFileUrl` path (full FLAC download, then `play_data`).
    pub async fn play_track(
        &self,
        client: &QobuzClient,
        track_id: u64,
        quality: Quality,
        start_position_secs: u64,
    ) -> Result<(), String> {
        // Supersede any earlier in-flight play_track for a different intent.
        let gen = self.begin_play();
        log::info!(
            "Player: Starting playback for track {} with quality {:?} (start {}s, gen {gen})",
            track_id,
            quality,
            start_position_secs
        );

        // Cache hit: replay instantly from L1/L2 unless the cached copy is
        // a lower quality than now requested.
        if let Some(cached) = self.audio_cache.get(track_id) {
            if cached_quality_below_requested(&cached.data, quality) {
                log::info!(
                    "[CACHE] Track {} cached below requested {:?} — re-fetching",
                    track_id,
                    quality
                );
            } else {
                if !self.is_current_play(gen) {
                    log::info!(
                        "Player: cache-hit play for track {track_id} superseded (gen {gen})"
                    );
                    return Ok(());
                }
                log::info!(
                    "[CACHE HIT] Track {} ({} bytes) — playing from cache",
                    track_id,
                    cached.size_bytes
                );
                // Use apply_play_data so we do not bump generation again.
                let r = self.apply_play_data(cached.data, track_id);
                // Cached tracks play from in-memory data (no streaming resume
                // offset); honor a session-resume position with a best-effort
                // seek once playback has been handed to the audio thread.
                if r.is_ok() && start_position_secs > 0 && self.is_current_play(gen) {
                    let _ = self.seek(start_position_secs);
                }
                return r;
            }
        }

        // `streaming_only` suppresses writing the track into the cache.
        let skip_cache = self
            .audio_settings
            .lock()
            .map(|s| s.streaming_only)
            .unwrap_or(false);

        // Try CMAF streaming pipeline first.
        // Only the init segment is fetched synchronously; audio segments
        // stream in a background task.
        log::info!("[CMAF] Attempting CMAF streaming for track {}", track_id);
        match qbz_qobuz::cmaf::setup_streaming(client, track_id, quality).await {
            Ok(cmaf_info) => {
                return self
                    .play_track_cmaf(cmaf_info, track_id, gen, skip_cache, start_position_secs)
                    .await;
            }
            Err(e) => {
                log::warn!(
                    "[CMAF] Streaming setup failed: {}, falling back to legacy download",
                    e
                );
                // Fall through to the legacy download path.
            }
        }

        self.play_track_legacy(client, track_id, quality, gen, skip_cache, start_position_secs)
            .await
    }
}
