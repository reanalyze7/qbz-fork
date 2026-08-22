use serde::{Deserialize, Serialize};

/// Error types for remote metadata operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "message")]
pub enum RemoteMetadataError {
    /// Network or connection error
    NetworkError(String),
    /// Rate limit exceeded
    RateLimited(String),
    /// Invalid response from provider
    InvalidResponse(String),
    /// No results found
    NoResults,
    /// Provider not available
    ProviderUnavailable(String),
    /// Invalid provider ID
    InvalidProviderId(String),
}

impl std::fmt::Display for RemoteMetadataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NetworkError(msg) => write!(f, "Network error: {}", msg),
            Self::RateLimited(msg) => write!(f, "Rate limited: {}", msg),
            Self::InvalidResponse(msg) => write!(f, "Invalid response: {}", msg),
            Self::NoResults => write!(f, "No results found"),
            Self::ProviderUnavailable(msg) => write!(f, "Provider unavailable: {}", msg),
            Self::InvalidProviderId(msg) => write!(f, "Invalid provider ID: {}", msg),
        }
    }
}

impl From<RemoteMetadataError> for String {
    fn from(err: RemoteMetadataError) -> Self {
        err.to_string()
    }
}
