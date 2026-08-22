//! `StreamFetcher`: reusable "download bytes reliably" primitive.
//!
//! Creates a fresh HTTP client per download to avoid HTTP/2 connection pool
//! poisoning: when a CDN connection breaks mid-transfer, a persistent client's
//! pool can keep reusing the dead connection, causing all subsequent downloads
//! to fail after 1 byte. Ephemeral clients guarantee a clean connection pool.
//!
//! Split into `fetch_file` (the retrying to-disk path), `try_download` (a
//! single streaming attempt), and `fetch_memory` (to-memory path) — all as
//! `impl StreamFetcher` blocks in sibling files.

mod fetch_file;
mod fetch_memory;
mod try_download;

use std::time::Duration;

/// Maximum number of download retry attempts
pub(super) const MAX_RETRIES: u32 = 3;

/// Backoff durations for each retry attempt
pub(super) const RETRY_BACKOFFS: [Duration; 3] = [
    Duration::from_secs(1),
    Duration::from_secs(3),
    Duration::from_secs(5),
];

pub struct StreamFetcher;

impl StreamFetcher {
    pub fn new() -> Self {
        Self
    }

    /// Build a fresh reqwest::Client for a single download.
    ///
    /// Each download gets its own client to prevent HTTP/2 connection pool
    /// poisoning from affecting subsequent downloads.
    pub(super) fn build_client() -> Result<reqwest::Client, String> {
        // rustls via the workspace reqwest (same TLS backend qbz-qobuz uses
        // for all CDN traffic, including CMAF segment downloads).
        reqwest::Client::builder()
            .timeout(Duration::from_secs(300)) // 5 minute timeout for large files
            .connect_timeout(Duration::from_secs(15))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))
    }
}

impl Default for StreamFetcher {
    fn default() -> Self {
        Self::new()
    }
}
