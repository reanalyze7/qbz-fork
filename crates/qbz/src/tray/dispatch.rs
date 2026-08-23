//! Player-action dispatch — mirrors the now-playing bar's handlers
//! (main.rs on_toggle_play / on_next / on_previous) so the tray drives the
//! exact same local-playback path.

use crate::AppWindow;

use super::Runtime;

pub(crate) fn dispatch_play_pause(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
) {
    let spawn_handle = handle.clone();
    handle.spawn(async move {
        crate::playback::toggle_play_pause(runtime, weak, spawn_handle);
    });
}

pub(crate) fn dispatch_next(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
) {
    let spawn_handle = handle.clone();
    handle.spawn(async move {
        crate::playback::next(runtime, weak, spawn_handle);
    });
}

pub(crate) fn dispatch_previous(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
) {
    let spawn_handle = handle.clone();
    handle.spawn(async move {
        crate::playback::previous(runtime, weak, spawn_handle);
    });
}

/// Step the local volume by `ticks` notches of 5% (positive = up). Mirrors the
/// Tauri `tray:volume_delta` handler. Local-only for now (remote-renderer
/// volume forwarding is a later refinement). Linux-only: scroll-to-volume is a
/// StatusNotifierItem feature the macOS/Windows tray doesn't expose.
#[cfg(target_os = "linux")]
pub(crate) fn dispatch_volume_delta(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    ticks: i32,
) {
    let spawn_handle = handle.clone();
    handle.spawn(async move {
        let current = runtime.core().player().get_playback_event().volume;
        let next = (current + ticks as f32 * 0.05).clamp(0.0, 1.0);
        crate::playback::set_volume(runtime, weak, spawn_handle, next);
    });
}
