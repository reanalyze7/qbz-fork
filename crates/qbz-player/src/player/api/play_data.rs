use super::*;

impl Player {
    /// Play from raw audio data (for cached tracks)
    /// Play from raw audio data (for cached / offline tracks).
    ///
    /// Bumps play generation so this call supersedes any in-flight
    /// `play_track` that has not yet applied audio.
    pub fn play_data(&self, data: Vec<u8>, track_id: u64) -> Result<(), String> {
        let _gen = self.begin_play();
        self.apply_play_data(data, track_id)
    }

    /// Send `Play` without bumping generation (used by `play_track` after
    /// its own `begin_play` + supersede checks).
    pub(crate) fn apply_play_data(&self, data: Vec<u8>, track_id: u64) -> Result<(), String> {
        log::info!(
            "Player: Playing {} bytes of audio data for track {}",
            data.len(),
            track_id
        );

        // Extract audio metadata (sample rate, channels, bit depth) - fast header-only read
        let meta = extract_audio_metadata_full(&data)
            .map_err(|e| format!("Failed to extract audio metadata: {}", e))?;

        let sample_rate = meta.sample_rate;
        let channels = meta.channels;
        let bit_depth = meta.bit_depth.unwrap_or(16);

        log::info!(
            "Player: Detected audio format - {}Hz, {} channels, {}-bit",
            sample_rate,
            channels,
            bit_depth
        );

        // Update shared state with actual stream quality
        self.state.set_stream_quality(sample_rate, bit_depth);

        self.tx
            .send(AudioCommand::Play {
                data,
                track_id,
                duration_secs: 0, // Will be determined by decoder
                sample_rate,
                channels,
            })
            .map_err(|e| {
                log::error!("Player: Failed to send to audio thread: {}", e);
                format!(
                    "Failed to send play command (audio thread may have crashed): {}",
                    e
                )
            })?;

        log::info!("Player: Playback initiated successfully");
        Ok(())
    }

}
