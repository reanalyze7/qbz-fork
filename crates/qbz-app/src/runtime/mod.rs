//! Runtime Session Contract
//!
//! Implements the UI-agnostic runtime lifecycle as per ADR_RUNTIME_SESSION_CONTRACT.md
//!
//! Key concepts:
//! - Single bootstrap entrypoint
//! - Canonical state machine (Uninitialized -> Ready)
//! - Typed errors (no string matching)
//! - Command gating in backend
//! - Lifecycle events

mod error;
mod manager;
mod requirement;
#[cfg(test)]
mod tests;
mod transitions;

use serde::{Deserialize, Serialize};

pub use error::RuntimeError;
pub use manager::RuntimeManager;
pub use requirement::CommandRequirement;

/// Canonical runtime states
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", content = "data")]
pub enum RuntimeState {
    /// Initial state - nothing initialized
    Uninitialized,
    /// Client initialized but no authentication
    InitializedNoAuth,
    /// Authenticated but per-user session not activated (transitional)
    AuthenticatedNoUserSession { user_id: u64 },
    /// Fully ready - all systems operational
    Ready { user_id: u64 },
    /// Degraded state - something is broken
    Degraded { reason: DegradedReason },
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self::Uninitialized
    }
}

/// Reasons for degraded state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "code", content = "message")]
pub enum DegradedReason {
    /// Bundle token extraction failed
    BundleExtractionFailed(String),
    /// CoreBridge initialization failed
    CoreBridgeInitFailed(String),
    /// Network connectivity issues
    NetworkError(String),
    /// Database/storage issues
    StorageError(String),
    /// Session activation failed (per-user stores not initialized)
    SessionActivationFailed(String),
}

/// Full runtime status returned by runtime_get_status and runtime_bootstrap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStatus {
    /// Current state
    pub state: RuntimeState,
    /// User ID if authenticated (None if not logged in)
    pub user_id: Option<u64>,
    /// Whether the API client is initialized (bundle tokens extracted)
    pub client_initialized: bool,
    /// Whether legacy auth is active
    pub legacy_auth: bool,
    /// Whether CoreBridge/V2 auth is active
    pub corebridge_auth: bool,
    /// Whether per-user session is activated
    pub session_activated: bool,
    /// Degraded reason if state is Degraded
    pub degraded_reason: Option<DegradedReason>,
}

impl Default for RuntimeStatus {
    fn default() -> Self {
        Self {
            state: RuntimeState::Uninitialized,
            user_id: None,
            client_initialized: false,
            legacy_auth: false,
            corebridge_auth: false,
            session_activated: false,
            degraded_reason: None,
        }
    }
}

/// Events emitted during runtime lifecycle
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", content = "data")]
pub enum RuntimeEvent {
    /// A live (cold) bundle-token extraction has started — no cache was
    /// available, so we must download Qobuz's ~7 MB bundle before the UI can
    /// proceed. The frontend shows a "connecting to Qobuz" state. Only emitted
    /// on a cold start (first run or after a cache wipe); warm starts skip it.
    BundleFetchStarted,
    /// Runtime initialized (client ready)
    RuntimeInitialized,
    /// Authentication state changed
    AuthChanged {
        logged_in: bool,
        user_id: Option<u64>,
    },
    /// Per-user session activated
    UserSessionActivated { user_id: u64 },
    /// Per-user session deactivated
    UserSessionDeactivated,
    /// CoreBridge auth failed
    CoreBridgeAuthFailed { error: String },
    /// Runtime entered degraded state
    RuntimeDegraded { reason: DegradedReason },
    /// Runtime fully ready
    RuntimeReady { user_id: u64 },
}
