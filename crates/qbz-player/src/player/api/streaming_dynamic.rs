use super::*;

impl Player {
    /// Play from streaming source with dynamic buffer based on measured speed.
    /// `start_position_secs` > 0 signals session resume (see `play_streaming`).
    pub fn play_streaming_dynamic(
        &self,
        track_id: u64,
        sample_rate: u32,
        channels: u16,
        bit_depth: u32,
        content_length: u64,
        speed_mbps: f64,
        duration_secs: u64,
        start_position_secs: u64,
    ) -> Result<BufferWriter, String> {
        let _gen = self.begin_play();
        self.apply_play_streaming_dynamic(
            track_id,
            sample_rate,
            channels,
            bit_depth,
            content_length,
            speed_mbps,
            duration_secs,
            start_position_secs,
        )
    }

    /// `play_streaming_dynamic` without bumping generation (used by
    /// `play_track`'s CMAF path, which already holds a generation token; a
    /// mid-intent bump here would invalidate a strictly newer play that
    /// started in the meantime).
    pub(crate) fn apply_play_streaming_dynamic(
        &self,
        track_id: u64,
        sample_rate: u32,
        channels: u16,
        bit_depth: u32,
        content_length: u64,
        speed_mbps: f64,
        duration_secs: u64,
        start_position_secs: u64,
    ) -> Result<BufferWriter, String> {
        log::info!(
            "Player: Starting dynamic streaming for track {} ({}Hz, {}ch, {}-bit, {:.2} MB, {:.1} MB/s, {}s, start={}s)",
            track_id,
            sample_rate,
            channels,
            bit_depth,
            content_length as f64 / (1024.0 * 1024.0),
            speed_mbps,
            duration_secs,
            start_position_secs
        );

        // Update shared state with actual stream quality
        self.state.set_stream_quality(sample_rate, bit_depth);

        // Use StreamingConfig::from_speed_mbps for dynamic buffer sizing
        let mut config = StreamingConfig::from_speed_mbps(speed_mbps);

        // Floor the initial buffer at the user's `stream_buffer_seconds` worth
        // of REAL audio bytes (#591). content_length / duration is the track's
        // true average byterate (both derive from the CMAF segment table),
        // unlike `speed_mbps`, which is estimated from the tiny init fetch and
        // is latency-dominated — it lands on the slowest ladder rung for every
        // connection. Clamped to 256KB (format-detection minimum) .. 8MB (the
        // process-wide ladder cap has no desktop caller, so this is the
        // effective ceiling protecting low-memory hosts).
        let user_secs = self
            .audio_settings
            .lock()
            .map(|s| s.stream_buffer_seconds)
            .unwrap_or(2);
        let bps = content_length / duration_secs.max(1);
        let floor = (user_secs as u64).saturating_mul(bps) as usize;
        let ladder_bytes = config.initial_buffer_bytes;
        config.initial_buffer_bytes = ladder_bytes
            .max(floor)
            .clamp(256 * 1024, 8 * 1024 * 1024);
        if config.initial_buffer_bytes != ladder_bytes {
            log::info!(
                "Dynamic buffer: user floor {}s x {} B/s → {}KB initial buffer (ladder gave {}KB)",
                user_secs,
                bps,
                config.initial_buffer_bytes / 1024,
                ladder_bytes / 1024
            );
        }

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

        log::info!("Player: Dynamic streaming playback initiated");
        Ok(writer)
    }
}
