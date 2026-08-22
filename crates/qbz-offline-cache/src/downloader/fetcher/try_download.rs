//! `StreamFetcher::try_download` — single-attempt streaming write with
//! progress-event throttling.

use std::io::Write;
use std::path::Path;
use std::time::{Duration, Instant};

use crate::event::{CacheEvent, CacheEventSink};

use super::super::validate::validate_download_size;
use super::StreamFetcher;

impl StreamFetcher {
    /// Single download attempt: stream response body to a temp file.
    pub(super) async fn try_download(
        &self,
        client: &reqwest::Client,
        url: &str,
        temp_path: &Path,
        track_id: u64,
        sink: Option<&CacheEventSink>,
    ) -> Result<u64, String> {
        let response = client
            .get(url)
            .header("User-Agent", "Mozilla/5.0")
            .send()
            .await
            .map_err(|e| format!("Failed to start fetch: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()));
        }

        let total_size = response.content_length();
        log::info!(
            "Caching started for track {}, total size: {:?} bytes",
            track_id,
            total_size
        );

        let mut file = std::fs::File::create(temp_path)
            .map_err(|e| format!("Failed to create temp file: {}", e))?;

        let mut cached: u64 = 0;
        let mut last_progress: u8 = 0;
        let mut last_emit_time = Instant::now();
        const MIN_EMIT_INTERVAL: Duration = Duration::from_millis(200);

        let mut stream = response.bytes_stream();
        use futures_util::StreamExt;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| {
                use std::error::Error as _;
                let mut msg = format!("Fetch error: {}", e);
                let mut source = e.source();
                while let Some(cause) = source {
                    msg.push_str(&format!(" | caused by: {}", cause));
                    source = cause.source();
                }
                log::error!(
                    "[Offline] Download error for track {} after {} bytes: {}",
                    track_id,
                    cached,
                    msg
                );
                msg
            })?;

            file.write_all(&chunk)
                .map_err(|e| format!("Failed to write chunk: {}", e))?;

            cached += chunk.len() as u64;

            // Calculate progress
            let progress = if let Some(total) = total_size {
                ((cached as f64 / total as f64) * 100.0) as u8
            } else {
                0
            };

            // Emit progress event every 2% change AND at least 200ms apart (always emit 100%)
            let elapsed = last_emit_time.elapsed();
            if progress != last_progress
                && (progress - last_progress >= 2 || progress == 100)
                && (elapsed >= MIN_EMIT_INTERVAL || progress == 100)
            {
                last_progress = progress;
                last_emit_time = Instant::now();

                if let Some(sink) = sink {
                    sink(CacheEvent::Progress {
                        track_id,
                        progress_percent: progress,
                        bytes_downloaded: cached,
                        total_bytes: total_size,
                    });
                }

                log::debug!(
                    "Caching progress for track {}: {}% ({}/{:?} bytes)",
                    track_id,
                    progress,
                    cached,
                    total_size
                );
            }
        }

        // Ensure all data is written
        file.flush()
            .map_err(|e| format!("Failed to flush file: {}", e))?;
        drop(file);

        validate_download_size(track_id, cached, total_size)?;

        Ok(cached)
    }
}
