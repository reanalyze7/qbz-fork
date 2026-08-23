//! Linux system tray via `ksni` (StatusNotifierItem).
//!
//! Faithful port of `src-tauri/src/tray_linux_ksni.rs`. The icon decoding,
//! theme resolution, tooltip composition and the updater-thread pattern are
//! byte-for-byte equivalent. The ONLY behavioural difference: tray actions
//! drive the playback controller + winit window directly (no webview to emit
//! events to) — see the `super::dispatch_*` / `super::*_window` helpers.

mod dark_mode;
mod icons;
mod init;
mod menu;
mod tray_impl;
mod updater;
mod updater_api;

pub use init::init;
pub use updater::LinuxTrayHandle;
