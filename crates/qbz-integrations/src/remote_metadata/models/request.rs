use serde::{Deserialize, Serialize};

use super::album::RemoteAlbumSearchResult;
use super::provider::RemoteProvider;

/// Search request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteSearchRequest {
    /// Provider to search
    pub provider: RemoteProvider,
    /// Search query (usually "artist album")
    pub query: String,
    /// Optional catalog number for more precise matching
    pub catalog_id: Option<String>,
    /// Optional artist name for filtering
    pub artist: Option<String>,
    /// Maximum results to return (default: 10)
    pub limit: Option<usize>,
}

impl RemoteSearchRequest {
    pub fn limit(&self) -> usize {
        self.limit.unwrap_or(10).min(25)
    }
}

/// Search response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteSearchResponse {
    /// Provider that was searched
    pub provider: RemoteProvider,
    /// Search results
    pub results: Vec<RemoteAlbumSearchResult>,
    /// Total results available (may be more than returned)
    pub total_count: Option<usize>,
    /// Whether rate limit was hit
    pub rate_limited: bool,
}

impl RemoteSearchResponse {
    pub fn empty(provider: RemoteProvider) -> Self {
        Self {
            provider,
            results: Vec::new(),
            total_count: Some(0),
            rate_limited: false,
        }
    }

    pub fn rate_limited(provider: RemoteProvider) -> Self {
        Self {
            provider,
            results: Vec::new(),
            total_count: None,
            rate_limited: true,
        }
    }
}
