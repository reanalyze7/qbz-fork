//! Tray init entry point: the platform-gated setup and the process-global
//! `TRAY` handle.

use std::sync::atomic::{AtomicBool, Ordering};

use crate::AppWindow;

use super::handle::TrayHandle;
use super::Runtime;

#[cfg(target_os = "linux")]
use super::linux;
#[cfg(target_os = "macos")]
use super::macos;

/// Process-global tray handle, set once by `init`. `None` until the tray is
/// created (or forever, if disabled / unsupported platform).
static TRAY: std::sync::OnceLock<TrayHandle> = std::sync::OnceLock::new();

/// The live tray handle, if the tray was initialized. Callers (playback poll
/// loop, settings) use this to push tooltip / theme updates.
pub fn handle() -> Option<&'static TrayHandle> {
    TRAY.get()
}

/// Initialize the system tray, gated by the user's `enable_tray` setting.
/// `theme_override` is the persisted `tray_icon_theme` ("auto"/"mono-light"/
/// "mono-dark"/"color"). No-op when disabled or already initialized.
pub fn init(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    theme_override: String,
    enabled: bool,
) {
    if !enabled {
        log::info!("[tray] disabled by user setting");
        return;
    }
    if TRAY.get().is_some() {
        return;
    }
    // On Linux TRAY is set asynchronously (inside the init thread below), so
    // the OnceLock check alone leaves a window where a second shell entry
    // (offline session -> D2 recovery login) would spawn a duplicate ksni
    // tray. This synchronous flag closes it; checked after `enabled` so a
    // disabled first call does not burn the one-shot.
    static INIT_STARTED: AtomicBool = AtomicBool::new(false);
    if INIT_STARTED.swap(true, Ordering::SeqCst) {
        return;
    }

    #[cfg(target_os = "linux")]
    {
        // ksni's blocking `spawn()` calls `Runtime::block_on` internally, which
        // panics inside an existing tokio runtime (`init` is called from the
        // tokio-based shell-entry task). Run the ksni setup on a dedicated
        // std::thread, outside any tokio context (the Tauri build is safe
        // because it inits from the non-tokio Tauri setup hook). The ksni
        // service + updater thread persist independently; this thread exits.
        std::thread::Builder::new()
            .name("qbz-tray-init".into())
            .spawn(move || match linux::init(runtime, weak, handle, &theme_override) {
                Ok(linux_handle) => {
                    let _ = TRAY.set(TrayHandle {
                        linux: Some(linux_handle),
                    });
                }
                Err(e) => log::error!("[tray] Linux tray init failed: {e}"),
            })
            .expect("spawn tray init thread");
    }

    #[cfg(target_os = "macos")]
    {
        // The NSStatusItem is !Send and must be built on the main thread with
        // NSApplication already running — create it on the Slint event loop.
        let _ = slint::invoke_from_event_loop(move || {
            macos::create(runtime, weak, handle, &theme_override);
            let _ = TRAY.set(TrayHandle::default());
        });
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let _ = (runtime, weak, handle, theme_override);
        log::info!("[tray] no tray backend on this platform");
    }
}
