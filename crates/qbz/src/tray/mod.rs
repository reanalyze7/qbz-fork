//! System tray for the Slint app.
//!
//! Faithful port of the Tauri tray (`src-tauri/src/tray.rs` +
//! `src-tauri/src/tray_linux_ksni.rs`). Platform split, same as Tauri:
//!   - **Linux** → `ksni` / StatusNotifierItem (`linux` submodule). Tauri's
//!     libayatana path never dispatches primary-activate (left-click); ksni
//!     exposes Activate / SecondaryActivate / Scroll (issue #310).
//!   - **macOS / Windows** → `tray-icon` (added with the
//!     CustomApplicationHandler slice). Until then, a no-op on those targets.
//!
//! Tray actions differ from Tauri in ONE way: there is no webview to emit
//! events to, so play/pause/next/previous/volume call the playback controller
//! directly (mirroring the now-playing bar's dispatch) and
//! show/hide toggles the winit window in place.

use std::sync::Arc;

use qbz_app::shell::AppRuntime;

use crate::adapter::SlintAdapter;

mod dispatch;
mod handle;
mod init;
mod lifecycle;
mod window;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
mod macos;

/// Shared runtime handle type used across the Slint app.
pub(crate) type Runtime = Arc<AppRuntime<SlintAdapter>>;

pub use init::{handle, init};
pub(crate) use lifecycle::{quit, set_mac_dock_hidden};
pub(crate) use window::{hide_window, present, set_window_shown, toggle_window};

pub(crate) use dispatch::{dispatch_next, dispatch_play_pause, dispatch_previous};
#[cfg(target_os = "linux")]
pub(crate) use dispatch::dispatch_volume_delta;
