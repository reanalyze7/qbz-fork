//! Last.fm API client — struct, construction, and session-key bookkeeping.
//!
//! The actual endpoint implementations live in sibling modules
//! (`auth`, `scrobble`, `similarity_*`, `top_*`, `loved_tracks`,
//! `recent_tracks`, `json_helpers`) as additional `impl LastFmClient` blocks
//! and free helper functions.

use reqwest::Client;

/// Cloudflare Workers proxy URL - handles API credentials and signature generation
pub(super) const LASTFM_PROXY_URL: &str = "https://qbz-api-proxy.blitzkriegfc.workers.dev/lastfm";

/// Last.fm API client
///
/// Uses Cloudflare Workers proxy to handle API credentials and signature generation.
/// This means the client doesn't need to know the API key or secret.
pub struct LastFmClient {
    pub(super) client: Client,
    pub(super) session_key: Option<String>,
}

impl Default for LastFmClient {
    fn default() -> Self {
        Self::new()
    }
}

impl LastFmClient {
    /// Create a new Last.fm client
    pub fn new() -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::USER_AGENT,
            reqwest::header::HeaderValue::from_static("QBZ/1.0.0"),
        );

        Self {
            client: Client::builder()
                .default_headers(headers)
                .build()
                .unwrap_or_else(|_| Client::new()),
            session_key: None,
        }
    }

    /// Create a client with an existing session key
    pub fn with_session_key(session_key: String) -> Self {
        let mut client = Self::new();
        client.session_key = Some(session_key);
        client
    }

    /// Set the session key (for restoring a saved session)
    pub fn set_session_key(&mut self, key: String) {
        self.session_key = Some(key);
    }

    /// Get the current session key
    pub fn session_key(&self) -> Option<&str> {
        self.session_key.as_deref()
    }

    /// Check if authenticated
    pub fn is_authenticated(&self) -> bool {
        self.session_key.is_some()
    }

    /// Clear the session (logout)
    pub fn clear_session(&mut self) {
        self.session_key = None;
    }
}
