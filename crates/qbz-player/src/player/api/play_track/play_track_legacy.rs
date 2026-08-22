use super::super::*;

impl Player {
    /// Legacy fallback path for `play_track`: fetch a `/track/getFileUrl`
    /// stream URL, download the full file, cache it, and hand it to
    /// `play_data`. Used when CMAF streaming setup fails.
    pub(super) async fn play_track_legacy(
        &self,
        client: &QobuzClient,
        track_id: u64,
        quality: Quality,
        gen: u64,
        skip_cache: bool,
        start_position_secs: u64,
    ) -> Result<(), String> {
        if !self.is_current_play(gen) {
            log::info!(
                "Player: legacy path for track {track_id} superseded before URL fetch (gen {gen})"
            );
            return Ok(());
        }

        // Legacy fallback: get the stream URL
        log::info!("Player: Getting stream URL...");
        let stream_url = client
            .get_stream_url_with_fallback(track_id, quality)
            .await
            .map_err(|e| {
                log::error!("Player: Failed to get stream URL: {}", e);
                format!("Failed to get stream URL: {}", e)
            })?;

        if !self.is_current_play(gen) {
            log::info!(
                "Player: legacy download for track {track_id} superseded after URL (gen {gen})"
            );
            return Ok(());
        }

        log::info!(
            "Player: Got stream URL: {} (format: {})",
            stream_url.url,
            stream_url.mime_type
        );

        // Download the audio data
        log::info!("Player: Starting audio caching...");
        let audio_data = self.download_audio(&stream_url.url).await.map_err(|e| {
            log::error!("Player: Caching failed: {}", e);
            e
        })?;
        log::info!("Player: Cached {} bytes of audio data", audio_data.len());

        if !self.is_current_play(gen) {
            log::info!(
                "Player: legacy play for track {track_id} superseded after download (gen {gen})"
            );
            return Ok(());
        }

        // Store the legacy download in the cache for instant replay.
        if !skip_cache {
            self.audio_cache.insert(track_id, audio_data.clone());
        }

        // Send to audio thread (do not re-bump generation)
        let r = self.apply_play_data(audio_data, track_id);
        if r.is_ok() && start_position_secs > 0 && self.is_current_play(gen) {
            let _ = self.seek(start_position_secs);
        }
        r
    }
}
