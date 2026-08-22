use super::*;

mod segment;

impl Player {
    /// Stream CMAF segments to the player's buffer, decrypting on the fly.
    ///
    /// Writes the FLAC header first so the decoder can identify the format,
    /// then fetches each audio segment, decrypts encrypted frames, and pushes
    /// the resulting FLAC frame data to the streaming buffer. The player
    /// starts playing as soon as enough data is buffered.
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn cmaf_stream_segments(
        url_template: &str,
        n_segments: u8,
        content_key: [u8; 16],
        flac_header: Vec<u8>,
        writer: BufferWriter,
        track_id: u64,
        cache: Arc<qbz_cache::AudioCache>,
        skip_cache: bool,
    ) -> Result<(), String> {
        struct FailGuard {
            writer: BufferWriter,
            armed: bool,
        }
        impl Drop for FailGuard {
            fn drop(&mut self) {
                if self.armed {
                    let _ = self
                        .writer
                        .error("CMAF stream aborted before completion".into());
                }
            }
        }
        let mut guard = FailGuard {
            writer,
            armed: true,
        };
        let writer = &guard.writer;
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| format!("CMAF client error: {}", e))?;

        if let Err(e) = writer.push_chunk(&flac_header) {
            let msg = format!("Failed to write FLAC header to buffer: {e}");
            let _ = writer.error(msg.clone());
            return Err(msg);
        }

        let mut total_written: u64 = flac_header.len() as u64;
        let mut cache_data: Vec<u8> = if skip_cache {
            Vec::new()
        } else {
            flac_header.clone()
        };
        let start = Instant::now();

        for seg_idx in 1..=n_segments {
            segment::fetch_and_push_segment(
                &client,
                url_template,
                seg_idx,
                content_key,
                writer,
                skip_cache,
                &mut cache_data,
                &mut total_written,
            )
            .await?;

            if seg_idx % 5 == 0 || seg_idx == n_segments {
                let elapsed = start.elapsed().as_secs_f64();
                let mbps = if elapsed > 0.0 {
                    total_written as f64 / (1024.0 * 1024.0) / elapsed
                } else {
                    0.0
                };
                qbz_audio::network_throttle::state().record_segment_bandwidth(mbps);
                log::info!(
                    "[CMAF-STREAM] Segment {}/{} ({:.1} MB, {:.1} MB/s)",
                    seg_idx,
                    n_segments - 1,
                    total_written as f64 / (1024.0 * 1024.0),
                    mbps
                );
            }
        }

        // Signal end of stream (disarm fail-guard so Drop does not error).
        guard.armed = false;
        if let Err(e) = writer.complete() {
            log::error!("[CMAF-STREAM] Failed to mark buffer complete: {}", e);
            let _ = writer.error(format!("Failed to mark buffer complete: {e}"));
            return Err(format!("Failed to mark buffer complete: {e}"));
        }

        log::info!(
            "[CMAF-STREAM] Complete: {:.2} MB written in {:.1}s for track {}, segments fetched: 1..{}",
            total_written as f64 / (1024.0 * 1024.0),
            start.elapsed().as_secs_f64(),
            track_id,
            n_segments - 1
        );

        if !skip_cache && !cache_data.is_empty() {
            let bytes = cache_data.len();
            cache.insert(track_id, cache_data);
            log::info!("[CMAF-STREAM] Track {} cached ({} bytes)", track_id, bytes);
        }

        Ok(())
    }
}
