//! Discogs API client for fetching album artwork
//!
//! Uses Cloudflare Workers proxy to search the Discogs database and download cover images.
//!
//! Endpoint logic lives in sibling modules: `artwork_fetch` (artwork-picker
//! backend's simple search+download path), `artwork_options` (multi-image
//! artwork picker), `download` (shared image download + the URL-based
//! download entry point), `search`/`metadata` (tag-editor lookups), `hash`
//! (cache filename hashing), and `types` (all DTOs).

use reqwest::Client;
use std::time::Duration;

mod artwork_fetch;
mod artwork_options;
mod artwork_options_helpers;
mod download;
mod hash;
mod metadata;
mod release_details;
mod search;
mod types;

pub use types::{
    DiscogsArtist, DiscogsImageOption, DiscogsLabel, DiscogsReleaseMetadata,
    DiscogsSearchResultExtended, DiscogsTrack, SearchResponse, SearchResult,
};

// Cloudflare Workers proxy URL - handles credentials
pub(super) const DISCOGS_PROXY_URL: &str = "https://qbz-api-proxy.blitzkriegfc.workers.dev/discogs";

/// Discogs API client
pub struct DiscogsClient {
    pub(super) client: Client,
}

impl DiscogsClient {
    /// Create a new Discogs client (proxy handles credentials)
    pub fn new() -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::USER_AGENT,
            reqwest::header::HeaderValue::from_static("Qoqobuz/1.0.0"),
        );

        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .default_headers(headers)
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    /// Check if credentials are configured (always true - proxy handles credentials)
    pub fn has_credentials(&self) -> bool {
        true
    }
}

impl Default for DiscogsClient {
    fn default() -> Self {
        Self::new()
    }
}
