// crates/qbzd/src/api/status/ — `GET /api/status` composite contract
// (02-cli-and-api.md §3.3.3; memo D14). Struct shape only in T2 — the
// `auth`/`qconnect`/`last_errors` sections already read from `DaemonShared`;
// `audio`/`playback`/`network` are placeholders wired by T3 (audio/playback
// driver), T6 (HTTP server + network reachability) and T10 (qconnect report
// tick, already flowing through `DaemonShared::qconnect`).
mod assemble;
mod device_cache;
mod handlers;
mod labels;
#[cfg(test)]
mod tests;

use serde::Serialize;

use crate::state::{AuthState, LatchedErrors};

pub use handlers::{info, status};

#[derive(Debug, Clone, Serialize)]
pub struct StatusDoc {
    pub version: String,
    pub api_version: u32,
    pub uptime_secs: u64,
    pub data_root: String,
    pub driver_tick_age_ms: Option<u64>,
    pub auth: AuthStatus,
    pub audio: AudioStatus,
    pub playback: PlaybackStatus,
    pub network: NetworkStatus,
    pub last_errors: LatchedErrors,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthStatus {
    pub state: AuthState,
    pub user_id: Option<u64>,
    pub subscription: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AudioStatus {
    pub backend: Option<String>,
    pub configured_device: Option<String>,
    pub device_present: bool,
    pub device_open: bool,
    /// `BitPerfectMode` serde variants: "DirectHardware"|"PluginFallback"|"Disabled"
    /// (crates/qbz-audio/src/backend.rs:226-233). Kept as a plain string here so
    /// this crate does not need to depend on the exact qbz-audio enum shape yet.
    pub bit_perfect: Option<String>,
    pub sample_rate: Option<u32>,
    pub bit_depth: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlaybackStatus {
    pub state: String,
    pub track_id: Option<u64>,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub position: Option<u64>,
    pub duration: Option<u64>,
    pub volume: f32,
    pub muted: bool,
    pub queue_len: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct NetworkStatus {
    pub online: bool,
}
