//! Linux tray init entry point.

use ksni::blocking::TrayMethods;

use super::super::Runtime;
use super::dark_mode::is_flatpak;
use super::icons::decode_tray_icons;
use super::tray_impl::QbzTray;
use super::updater::LinuxTrayHandle;
use crate::AppWindow;

/// Initialize the Linux ksni tray service. Spawns a background thread that owns
/// the SNI service and returns a cloneable handle for live tooltip / theme
/// updates. `theme_override`: "auto"/"mono-light"/"mono-dark"/"color".
pub fn init(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    theme_override: &str,
) -> Result<LinuxTrayHandle, Box<dyn std::error::Error>> {
    log::info!(
        "Initializing ksni tray (Linux, SNI primary-activate enabled, theme={:?})",
        theme_override
    );

    let icons = decode_tray_icons(Some(theme_override))?;
    let tray = QbzTray {
        runtime,
        weak,
        handle,
        icons,
        now_playing: None,
        is_playing: false,
    };

    // Flatpak requires disabling the well-known DBus name because the sandbox
    // cannot own arbitrary bus names.
    let ksni_handle = if is_flatpak() {
        log::info!("[tray] Flatpak detected — spawning ksni without DBus well-known name");
        tray.disable_dbus_name(true).spawn()?
    } else {
        tray.spawn()?
    };

    let live = LinuxTrayHandle::empty();
    live.install(ksni_handle);

    log::info!("ksni tray initialized");
    Ok(live)
}
