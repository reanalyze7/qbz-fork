//! `StreamFetcher::fetch_to_file` — retrying, to-disk download.

use std::path::Path;

use crate::event::CacheEventSink;

use super::{StreamFetcher, MAX_RETRIES, RETRY_BACKOFFS};

impl StreamFetcher {
    /// Fetch a stream and cache it to disk with progress updates.
    ///
    /// Retries up to MAX_RETRIES times with exponential backoff on transient
    /// failures (connection reset, EOF, timeout). Each retry creates a fresh
    /// HTTP client to avoid reusing a poisoned connection pool.
    pub async fn fetch_to_file(
        &self,
        url: &str,
        dest_path: &Path,
        track_id: u64,
        sink: Option<&CacheEventSink>,
    ) -> Result<u64, String> {
        log::info!("Caching track {} to {:?}", track_id, dest_path);

        // Create parent directories if needed
        if let Some(parent) = dest_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        let temp_path = dest_path.with_extension("tmp");

        let mut last_error = String::new();
        for attempt in 0..=MAX_RETRIES {
            if attempt > 0 {
                let backoff = RETRY_BACKOFFS[(attempt - 1) as usize];
                log::info!(
                    "[Offline] Retry {}/{} for track {} after {}s",
                    attempt,
                    MAX_RETRIES,
                    track_id,
                    backoff.as_secs()
                );
                tokio::time::sleep(backoff).await;
            }

            // Fresh client per attempt — prevents connection pool poisoning
            let client = Self::build_client()?;

            match self
                .try_download(&client, url, &temp_path, track_id, sink)
                .await
            {
                Ok(size) => {
                    // Move temp file to final destination
                    std::fs::rename(&temp_path, dest_path)
                        .map_err(|e| format!("Failed to move temp file: {}", e))?;
                    log::info!("Caching complete for track {}: {} bytes", track_id, size);
                    return Ok(size);
                }
                Err(e) => {
                    last_error = e;
                    // Clean up partial temp file before retry
                    let _ = std::fs::remove_file(&temp_path);
                    if attempt < MAX_RETRIES {
                        log::warn!(
                            "[Offline] Download attempt {} failed for track {}: {}",
                            attempt + 1,
                            track_id,
                            last_error
                        );
                    }
                }
            }
        }

        Err(last_error)
    }
}
