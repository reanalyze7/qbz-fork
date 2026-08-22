//! MusicBrainz API client
//!
//! HTTP client with rate limiting and proper User-Agent handling.
//! Uses Cloudflare Workers proxy for consistent rate limiting.

use std::time::{Duration, Instant};
use tokio::sync::Mutex;

mod areas;
mod areas_country;
mod areas_resolve;
mod artists;
mod artists_tag;
mod core;
mod http;
mod recordings;
mod releases;

pub use core::{MusicBrainzClient, MusicBrainzConfig};

/// Proxy URL for MusicBrainz requests
const MUSICBRAINZ_PROXY_URL: &str = "https://qbz-api-proxy.blitzkriegfc.workers.dev/musicbrainz";

/// Direct MusicBrainz API URL (fallback)
const MUSICBRAINZ_API_URL: &str = "https://musicbrainz.org/ws/2";

/// Rate limiter for MusicBrainz API
pub struct RateLimiter {
    last_request: Mutex<Instant>,
    min_interval: Duration,
}

impl RateLimiter {
    /// Create rate limiter for direct MusicBrainz API (1 req/sec)
    pub fn new() -> Self {
        Self::with_interval(Duration::from_millis(1100))
    }

    /// Create rate limiter for proxy (faster, proxy handles actual rate limiting)
    pub fn for_proxy() -> Self {
        Self::with_interval(Duration::from_millis(200))
    }

    /// Create rate limiter with custom interval
    pub fn with_interval(min_interval: Duration) -> Self {
        Self {
            // Start in the past so first request doesn't wait
            last_request: Mutex::new(Instant::now() - Duration::from_secs(2)),
            min_interval,
        }
    }

    pub async fn wait(&self) {
        let mut last = self.last_request.lock().await;
        let elapsed = last.elapsed();
        if elapsed < self.min_interval {
            tokio::time::sleep(self.min_interval - elapsed).await;
        }
        *last = Instant::now();
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}
