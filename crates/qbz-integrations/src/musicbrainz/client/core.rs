use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use super::{RateLimiter, MUSICBRAINZ_API_URL, MUSICBRAINZ_PROXY_URL};

/// MusicBrainz API client configuration
#[derive(Debug, Clone)]
pub struct MusicBrainzConfig {
    /// Whether MusicBrainz integration is enabled
    pub enabled: bool,
    /// Use proxy instead of direct API
    pub use_proxy: bool,
}

impl Default for MusicBrainzConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            // Direct-to-MusicBrainz by default: each client uses its OWN IP, so
            // the per-IP 1 req/s budget is per-user instead of shared across all
            // QBZ users behind the proxy's Cloudflare egress IPs (which is what
            // triggered the 503s). MB read access needs no key, so the proxy
            // added only the funnel. Flip to true to route via the proxy again.
            use_proxy: false,
        }
    }
}

/// MusicBrainz API client
pub struct MusicBrainzClient {
    pub(super) client: Client,
    pub(super) rate_limiter: Arc<RateLimiter>,
    pub(super) config: Arc<Mutex<MusicBrainzConfig>>,
}

impl Default for MusicBrainzClient {
    fn default() -> Self {
        Self::new()
    }
}

impl MusicBrainzClient {
    /// Create a new MusicBrainz client with default config
    pub fn new() -> Self {
        Self::with_config(MusicBrainzConfig::default())
    }

    /// Create client with specific configuration
    pub fn with_config(config: MusicBrainzConfig) -> Self {
        let version = "1.0.0";
        let user_agent = format!(
            "Qoqobuz/{} (https://github.com/reanalyze7/qbz-fork)",
            version
        );

        let client = Client::builder()
            .user_agent(&user_agent)
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| Client::new());

        // Use faster rate limiter when using proxy
        let rate_limiter = if config.use_proxy {
            RateLimiter::for_proxy()
        } else {
            RateLimiter::new()
        };

        Self {
            client,
            rate_limiter: Arc::new(rate_limiter),
            config: Arc::new(Mutex::new(config)),
        }
    }

    /// Check if MusicBrainz integration is enabled
    pub async fn is_enabled(&self) -> bool {
        self.config.lock().await.enabled
    }

    /// Enable or disable MusicBrainz integration
    pub async fn set_enabled(&self, enabled: bool) {
        self.config.lock().await.enabled = enabled;
    }

    /// Get the base URL based on configuration
    pub(super) async fn base_url(&self) -> &'static str {
        if self.config.lock().await.use_proxy {
            MUSICBRAINZ_PROXY_URL
        } else {
            MUSICBRAINZ_API_URL
        }
    }
}
