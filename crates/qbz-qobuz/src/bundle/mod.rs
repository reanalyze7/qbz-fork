//! Bundle token extraction from Qobuz web player
//!
//! Extracts app_id and secrets from the Qobuz JavaScript bundle.
//! This is necessary because Qobuz doesn't provide a public API.

use serde::{Deserialize, Serialize};
use std::time::Duration;

mod cache;
mod fetch;
mod parse;
mod refresh;
mod secrets;

pub use cache::load_cached_bundle;
pub use fetch::extract_bundle_tokens;
pub use refresh::{extract_and_cache_bundle_tokens, refresh_bundle_if_changed};

pub(crate) const LOGIN_PAGE_URL: &str = "https://play.qobuz.com/login";
pub(crate) const BUNDLE_BASE_URL: &str = "https://play.qobuz.com";

/// Per-request ceiling for the bundle fetch. The login page is tiny but the
/// bundle.js is ~7 MB and served from a CDN that is sometimes very slow; without
/// this, a stalled download blocks the entire app startup indefinitely.
pub(crate) const BUNDLE_FETCH_TIMEOUT: Duration = Duration::from_secs(45);
/// Extra attempts after the first on a failed/timed-out extraction.
pub(crate) const BUNDLE_EXTRACTION_RETRIES: usize = 2;

/// Extracted bundle tokens
#[derive(Debug, Clone)]
pub struct BundleTokens {
    pub app_id: String,
    pub secrets: Vec<String>,
    /// OAuth private key used for the /oauth/callback exchange.
    /// Present in recent bundle versions; None on older bundles.
    pub private_key: Option<String>,
}

/// On-disk cache of the extracted tokens, keyed by the Qobuz bundle version
/// (e.g. `8.1.0-b019`) so we can detect when Qobuz rotates the bundle and the
/// secrets change. Lives in the regenerable cache dir, never in precious data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedBundle {
    pub bundle_version: String,
    pub app_id: String,
    pub secrets: Vec<String>,
    #[serde(default)]
    pub private_key: Option<String>,
    /// Unix seconds when these tokens were fetched (freshness only; not a TTL).
    pub fetched_at: i64,
}

impl From<CachedBundle> for BundleTokens {
    fn from(c: CachedBundle) -> Self {
        BundleTokens {
            app_id: c.app_id,
            secrets: c.secrets,
            private_key: c.private_key,
        }
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
