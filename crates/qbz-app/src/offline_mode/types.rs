use serde::{Deserialize, Serialize};

use super::connectivity::Connectivity;

/// The app-level offline mode (derived; see [`super::OfflineModeEngine`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OfflineMode {
    Online,
    RealOffline,
    InducedOffline,
}

/// Full status broadcast to UIs on every change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfflineStatus {
    pub mode: OfflineMode,
    /// Raw connectivity, independent of the mode (the banner shows when an
    /// offline SESSION is active but connectivity is back).
    pub connectivity: Connectivity,
    /// Captive-portal hint from the prober.
    pub captive_portal: bool,
    /// The persisted induced flag (mirrors Settings).
    pub induced: bool,
    /// Session was started without Qobuz auth ("Start offline" from login).
    pub offline_session: bool,
}

impl OfflineStatus {
    pub fn is_offline(&self) -> bool {
        self.mode != OfflineMode::Online
    }

    /// D2: show the one-click login banner — an unauthenticated offline
    /// session while connectivity is actually up (and the user did not opt
    /// into induced offline).
    pub fn show_recovery_banner(&self) -> bool {
        self.offline_session && !self.induced && self.connectivity == Connectivity::Up
    }
}

pub(super) fn default_status() -> OfflineStatus {
    OfflineStatus {
        mode: OfflineMode::Online,
        connectivity: Connectivity::Unknown,
        captive_portal: false,
        induced: false,
        offline_session: false,
    }
}
