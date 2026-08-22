use super::*;

impl Player {
    /// Play from streaming source (starts playback before full download).
    /// Returns the BufferWriter so caller can push data as it downloads.
    /// `start_position_secs` > 0 turns this into a session-resume play
    /// (#315): the audio thread waits for enough buffer to cover the
    /// offset and pre-skips decoder output up to that point.
    pub fn play_streaming(
        &self,
        track_id: u64,
        sample_rate: u32,
        channels: u16,
        content_length: u64,
        buffer_seconds: u8,
        duration_secs: u64,
        start_position_secs: u64,
    ) -> Result<BufferWriter, String> {
        let _gen = self.begin_play();
        self.apply_play_streaming(
            track_id,
            sample_rate,
            channels,
            content_length,
            buffer_seconds,
            duration_secs,
            start_position_secs,
        )
    }

    /// `play_streaming` without bumping generation (used by play paths that
    /// already did their own `begin_play` + supersede checks, like
    /// `play_dsd_file`; a mid-intent bump here would invalidate a strictly
    /// newer play that started in the meantime).
    pub(crate) fn apply_play_streaming(
        &self,
        track_id: u64,
        sample_rate: u32,
        channels: u16,
        content_length: u64,
        buffer_seconds: u8,
        duration_secs: u64,
        start_position_secs: u64,
    ) -> Result<BufferWriter, String> {
        log::info!(
            "Player: Starting streaming playback for track {} ({}Hz, {}ch, {} bytes total, {}s, start={}s)",
            track_id,
            sample_rate,
            channels,
            content_length,
            duration_secs,
            start_position_secs
        );

        // Use StreamingConfig::from_seconds for proper buffer sizing
        let config = StreamingConfig::from_seconds(buffer_seconds);

        let (source, writer) = BufferedMediaSource::new(config, Some(content_length));
        let source = Arc::new(source);

        self.tx
            .send(AudioCommand::PlayStreaming {
                source: source.clone(),
                track_id,
                sample_rate,
                channels,
                duration_secs,
                start_position_secs,
                content_length,
                play_gen: self.state.current_play_generation(),
            })
            .map_err(|e| {
                log::error!("Player: Failed to send streaming command: {}", e);
                format!("Failed to send streaming play command: {}", e)
            })?;

        log::info!("Player: Streaming playback initiated");
        Ok(writer)
    }
}
