use super::*;

impl Player {
    /// Queue next track for gapless playback (appends to current Sink without stopping)
    pub fn play_next(&self, data: Vec<u8>, track_id: u64) -> Result<(), String> {
        let meta = extract_audio_metadata_full(&data)
            .map_err(|e| format!("Failed to extract audio metadata for gapless: {}", e))?;

        log::info!(
            "Player: Queueing gapless track {} ({}Hz, {}ch, {} bytes)",
            track_id,
            meta.sample_rate,
            meta.channels,
            data.len()
        );

        self.tx
            .send(AudioCommand::PlayNext {
                data,
                track_id,
                sample_rate: meta.sample_rate,
                channels: meta.channels,
            })
            .map_err(|e| {
                log::error!("Player: Failed to send PlayNext to audio thread: {}", e);
                format!("Failed to send gapless command: {}", e)
            })
    }
}
