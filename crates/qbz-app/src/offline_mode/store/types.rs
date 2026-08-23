use serde::{Deserialize, Serialize};

/// The offline-mode settings the port consumes.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfflineModeSettings {
    pub manual_offline_mode: bool,
    pub show_network_folders_in_manual_offline: bool,
}

/// One row of the Last.fm offline scrobble queue (`scrobble_queue`). Mirrors
/// Tauri's `offline::QueuedScrobble` — same table, same per-user file, so
/// scrobbles queued by one frontend flush from the other.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueuedScrobble {
    pub id: i64,
    pub artist: String,
    pub track: String,
    pub album: Option<String>,
    pub timestamp: i64,
    pub created_at: i64,
    pub sent: bool,
}
