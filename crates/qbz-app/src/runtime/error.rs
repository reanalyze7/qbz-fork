use serde::{Deserialize, Serialize};

use super::DegradedReason;

/// Typed runtime errors - no string matching in clients
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "code", content = "details")]
pub enum RuntimeError {
    /// Runtime not initialized - call runtime_bootstrap first
    RuntimeNotInitialized,
    /// Authentication required for this operation
    AuthRequired,
    /// Per-user session not activated - call activate_user_session
    UserSessionNotActivated,
    /// CoreBridge/V2 auth missing - V2 commands won't work
    CoreBridgeAuthMissing,
    /// Runtime is in degraded state
    RuntimeDegraded(DegradedReason),
    /// Invalid user ID (e.g., 0)
    InvalidUserId,
    /// Bootstrap already in progress
    BootstrapInProgress,
    /// V2 CoreBridge authentication failed
    V2AuthFailed(String),
    /// V2 CoreBridge not initialized
    V2NotInitialized,
    /// Manual offline mode is on and the requested track is not available
    /// in any local cache. Emitted instead of silently streaming from the
    /// network (see issue #279).
    TrackNotAvailableOffline,
    /// Internal error
    Internal(String),
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RuntimeNotInitialized => write!(f, "Runtime not initialized"),
            Self::AuthRequired => write!(f, "Authentication required"),
            Self::UserSessionNotActivated => write!(f, "User session not activated"),
            Self::CoreBridgeAuthMissing => write!(f, "CoreBridge authentication missing"),
            Self::RuntimeDegraded(reason) => write!(f, "Runtime degraded: {:?}", reason),
            Self::InvalidUserId => write!(f, "Invalid user ID"),
            Self::BootstrapInProgress => write!(f, "Bootstrap already in progress"),
            Self::V2AuthFailed(msg) => write!(f, "V2 authentication failed: {}", msg),
            Self::V2NotInitialized => write!(f, "V2 CoreBridge not initialized"),
            Self::TrackNotAvailableOffline => {
                write!(f, "Track not available in offline cache while in offline mode")
            }
            Self::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for RuntimeError {}
