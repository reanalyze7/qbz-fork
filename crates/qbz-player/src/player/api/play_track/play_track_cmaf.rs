use super::super::*;

impl Player {
    /// Handle a successful CMAF streaming setup: derive stream parameters
    /// from the init segment, start playback immediately via the streaming
    /// path, and spawn the background task that fetches/decrypts/pushes the
    /// remaining segments.
    pub(super) async fn play_track_cmaf(
        &self,
        cmaf_info: qbz_qobuz::cmaf::CmafStreamingInfo,
        track_id: u64,
        gen: u64,
        skip_cache: bool,
        start_position_secs: u64,
    ) -> Result<(), String> {
        if !self.is_current_play(gen) {
            log::info!("Player: CMAF setup for track {track_id} superseded (gen {gen})");
            return Ok(());
        }
        // Derive stream parameters from init segment metadata.
        let sample_rate = cmaf_info.sampling_rate.unwrap_or(44100);
        let channels = 2u16; // FLAC from Qobuz is always stereo
        let bit_depth = cmaf_info.bit_depth.unwrap_or(16);
        let total_flac_size = cmaf_info.flac_header.len() as u64
            + cmaf_info
                .segment_table
                .iter()
                .map(|s| s.byte_len as u64)
                .sum::<u64>();

        // Track duration from the CMAF segment table. The streaming path's
        // position timer clamps `current_position` to the duration it was
        // given, so a zero here freezes the seek bar at 0:00 and blocks
        // auto-advance — derive the real value from per-segment sample counts.
        let total_samples: u64 = cmaf_info
            .segment_table
            .iter()
            .map(|s| s.sample_count as u64)
            .sum();
        let duration_secs = if sample_rate > 0 {
            total_samples / sample_rate as u64
        } else {
            0
        };

        // Estimate speed from the init segment fetch (conservative: assume
        // ~10 MB/s if init was too fast to measure reliably).
        let speed_mbps = if cmaf_info.init_fetch_ms > 0 {
            let init_bytes = cmaf_info.flac_header.len() as f64 + 4096.0;
            (init_bytes / (cmaf_info.init_fetch_ms as f64 / 1000.0)) / (1024.0 * 1024.0)
        } else {
            10.0
        };

        log::info!(
            "[CMAF] Streaming setup: {}Hz, {}-bit, {:.2} MB total, {:.1} MB/s est, {} segments",
            sample_rate,
            bit_depth,
            total_flac_size as f64 / (1024.0 * 1024.0),
            speed_mbps,
            cmaf_info.n_segments
        );

        // Create the streaming buffer and start playback immediately.
        // Non-bumping variant: this intent already holds `gen`.
        let buffer_writer = self.apply_play_streaming_dynamic(
            track_id,
            sample_rate,
            channels,
            bit_depth,
            total_flac_size,
            speed_mbps,
            duration_secs,
            start_position_secs,
        )?;

        // Spawn the background task that fetches + decrypts + pushes audio
        // segments to the buffer.
        let url_template = cmaf_info.url_template.clone();
        let content_key = cmaf_info.content_key;
        let flac_header = cmaf_info.flac_header;
        let n_segments = cmaf_info.n_segments;
        let cache = self.audio_cache.clone();

        tokio::spawn(async move {
            match Self::cmaf_stream_segments(
                &url_template,
                n_segments,
                content_key,
                flac_header,
                buffer_writer,
                track_id,
                cache,
                skip_cache,
            )
            .await
            {
                Ok(()) => log::info!("[CMAF-STREAM COMPLETE] Track {}", track_id),
                Err(e) => log::error!("[CMAF-STREAM ERROR] Track {}: {}", track_id, e),
            }
        });

        Ok(())
    }
}
