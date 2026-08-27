//! ListenBrainz API client
//!
//! Direct client for ListenBrainz submissions (no proxy needed - uses user token)

use reqwest::Client;
use std::sync::Arc;
use tokio::sync::Mutex;

mod auth;
mod history;
mod identifier;
mod metadata;
mod playlist_tracks;
mod playlists;
mod recommendations;
mod releases;
mod submit;
mod submit_internal;

/// ListenBrainz API base URL
const LISTENBRAINZ_API_URL: &str = "https://api.listenbrainz.org/1";

/// ListenBrainz client configuration
#[derive(Debug, Clone)]
pub struct ListenBrainzConfig {
    /// Whether ListenBrainz integration is enabled
    pub enabled: bool,
    /// User token from listenbrainz.org
    pub token: Option<String>,
    /// Username (set after token validation)
    pub user_name: Option<String>,
}

impl Default for ListenBrainzConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            token: None,
            user_name: None,
        }
    }
}

/// ListenBrainz API client
pub struct ListenBrainzClient {
    client: Client,
    config: Arc<Mutex<ListenBrainzConfig>>,
    version: String,
}

impl Default for ListenBrainzClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ListenBrainzClient {
    /// Create a new ListenBrainz client
    pub fn new() -> Self {
        Self::with_config(ListenBrainzConfig::default())
    }

    /// Create client with specific configuration
    pub fn with_config(config: ListenBrainzConfig) -> Self {
        let version = "1.0.0".to_string();
        let user_agent = format!(
            "Qoqobuz/{} (https://github.com/reanalyze7/qbz-fork)",
            version
        );

        let client = Client::builder()
            .user_agent(&user_agent)
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            config: Arc::new(Mutex::new(config)),
            version,
        }
    }

    /// Set the application version for submission metadata
    pub fn set_version(&mut self, version: impl Into<String>) {
        self.version = version.into();
    }
}
