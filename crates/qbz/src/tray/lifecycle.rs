//! macOS Dock-icon policy + app quit — small tray lifecycle helpers split out
//! of `window.rs` to stay under the file-length limit.

#[cfg(target_os = "macos")]
use super::macos;

/// Apply the macOS Dock-icon activation policy (`.accessory` hides the Dock
/// icon, `.regular` keeps it). No-op off macOS. Must be called on the main
/// thread (it is, from the close handlers / window hide-show).
pub(crate) fn set_mac_dock_hidden(hidden: bool) {
    #[cfg(target_os = "macos")]
    macos::set_dock_icon_hidden(hidden);
    #[cfg(not(target_os = "macos"))]
    let _ = hidden;
}

/// Quit the whole app from a tray action (any thread).
pub(crate) fn quit() {
    log::info!("[tray] quit requested");
    let _ = slint::invoke_from_event_loop(|| {
        // Flush the session before the loop tears down.
        crate::session_persist::save_on_exit();
        let _ = slint::quit_event_loop();
    });
}
