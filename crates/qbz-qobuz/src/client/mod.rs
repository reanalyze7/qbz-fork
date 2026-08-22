//! Qobuz API client implementation
//!
//! The `QobuzClient` struct and its ~80 methods are split across this
//! directory by API domain (search, catalog reads, discover, tracks/artists,
//! playlists, labels, authenticated mutations, artist page, CMAF streaming).
//! Every method lives on the SAME inherent struct via multiple `impl
//! QobuzClient { ... }` blocks in different files — Rust allows this within
//! one crate, so callers see zero change: `client.method_name()` resolves
//! exactly as before.

use reqwest::Client;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::bundle::BundleTokens;
use super::error::Result;
use super::forbidden_breaker::ForbiddenBreaker;
use qbz_models::*;

mod accessors;
mod artist_page;
mod body_preview;
mod artist_pagination;
mod authenticated;
mod auth;
mod catalog_reads;
mod cmaf;
mod cmaf_session;
mod discover;
mod discover_extra;
mod dynamic_suggest;
mod favorite_mutations;
mod favorites;
mod forbidden_breaker_guard;
mod headers;
mod init;
mod labels;
mod labels_extra;
mod labels_list;
mod oauth;
mod oauth_token;
mod playlist_mutations;
mod playlist_paginated;
mod playlist_paginated_fetch;
mod playlist_reads;
mod playlist_subscribe;
mod release_watch;
mod search;
mod search_artists;
mod search_extra;
mod stream_fallback;
mod tracks_artists;

pub(crate) use body_preview::body_preview;

pub(crate) const USER_AGENT: &str =
    "Mozilla/5.0 (X11; Linux x86_64; rv:120.0) Gecko/20100101 Firefox/120.0";

/// CMAF session state (session/start + infos for key derivation)
struct CmafSession {
    session_id: String,
    infos: String,
    expires_at: u64,
}

/// Qobuz API client
pub struct QobuzClient {
    http: Client,
    tokens: Arc<RwLock<Option<BundleTokens>>>,
    session: Arc<RwLock<Option<UserSession>>>,
    validated_secret: Arc<RwLock<Option<String>>>,
    locale: Arc<RwLock<String>>,
    cmaf_session: Arc<RwLock<Option<CmafSession>>>,
    /// Backs off the hot streaming/favorites paths after repeated 403s so a
    /// post-outage account hiccup can't be escalated into a per-IP edge block
    /// by the no-backoff prefetch scheduler (issue #637).
    forbidden_breaker: Arc<ForbiddenBreaker>,
}

impl Clone for QobuzClient {
    fn clone(&self) -> Self {
        Self {
            http: self.http.clone(),
            tokens: Arc::clone(&self.tokens),
            session: Arc::clone(&self.session),
            validated_secret: Arc::clone(&self.validated_secret),
            locale: Arc::clone(&self.locale),
            cmaf_session: Arc::clone(&self.cmaf_session),
            forbidden_breaker: Arc::clone(&self.forbidden_breaker),
        }
    }
}

impl QobuzClient {
    /// Create a new client
    pub fn new() -> Result<Self> {
        let http = Client::builder()
            .user_agent(USER_AGENT)
            .cookie_store(true)
            // Bound the TCP connect phase so a dead route (e.g. a stale CDN
            // address) can't hang startup. Does not affect body-read time, so
            // long streaming reads are unaffected.
            .connect_timeout(std::time::Duration::from_secs(10))
            .build()?;

        Ok(Self {
            http,
            tokens: Arc::new(RwLock::new(None)),
            session: Arc::new(RwLock::new(None)),
            validated_secret: Arc::new(RwLock::new(None)),
            locale: Arc::new(RwLock::new("en".to_string())),
            cmaf_session: Arc::new(RwLock::new(None)),
            forbidden_breaker: Arc::new(ForbiddenBreaker::new()),
        })
    }
}

impl Default for QobuzClient {
    fn default() -> Self {
        Self::new().expect("Failed to create client")
    }
}

#[cfg(test)]
mod tests;
