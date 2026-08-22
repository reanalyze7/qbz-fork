//! Desktop "now playing" notifications for track changes (frontend-agnostic, ADR-006).
//!
//! 1:1 port of the Tauri notification path (`src-tauri/src/commands_v2/
//! legacy_compat.rs::v2_show_track_notification` + the artwork helpers in
//! `helpers.rs`), lifted out of the Tauri command layer so any frontend
//! (Slint / TUI) can fire it from native Rust instead of a webview `invoke`.
//!
//!   - **Linux** → XDG notification portal via `ashpd` (goes over D-Bus). The
//!     album art is passed as `Icon::Bytes(png)` — the portal rejects huge
//!     payloads, so the cover is center-cropped to a square, downscaled to
//!     <=512px, and re-encoded PNG (<=4 MiB).
//!   - **macOS** → `notify_rust` with `image_path` (it needs a file on disk, so
//!     the cover is cached but NOT resized).
//!   - **Windows** → not implemented (parity with Tauri).
//!
//! The whole thing is fire-and-forget: failures are logged, never surfaced, so
//! a missing portal or a slow CDN never blocks playback. The HTTP download +
//! image work run on `spawn_blocking` (a tokio runtime must be present — it is,
//! the app drives one).

mod artwork_cache;
mod format;
#[cfg(target_os = "linux")]
mod linux_icon;
mod show;

pub use show::show_track_notification;

/// Everything needed to render a track-change notification. The crate formats
/// the body + quality line itself so the output matches the Tauri notification
/// exactly, regardless of frontend.
#[derive(Debug, Clone, Default)]
pub struct NotificationMeta {
    pub title: String,
    pub artist: String,
    pub album: String,
    /// Bit depth (e.g. 16, 24). Drives the quality line.
    pub bit_depth: Option<u32>,
    /// Sample rate in kHz (e.g. 44.1, 96.0). Drives the quality line.
    pub sample_rate: Option<f64>,
    /// Album-art URL: http/https (downloaded + cached), `file://`, or
    /// `asset://localhost/...` (resolved to a local path). `None` = no art.
    pub art_url: Option<String>,
}
