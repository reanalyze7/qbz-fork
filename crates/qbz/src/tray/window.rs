//! Window show/hide — used by left-click, the "Show/Hide" menu item, and
//! close-to-tray. Uses Slint's own `hide()`/`show()` (NOT winit `set_visible`,
//! which is a no-op on Wayland): since Slint 1.7 `hide()` destroys the winit
//! surface on Wayland and `show()` recreates it (PR slint-ui/slint#5529), the
//! only path that actually works on KWin Wayland. The app survives a hidden
//! window because main() runs the loop via `run_event_loop_until_quit()`
//! (quit_on_last_window_closed = false).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Instant;

use slint::ComponentHandle;

use crate::AppWindow;

#[cfg(target_os = "macos")]
use super::macos;

/// Whether the main window is currently shown. The tray toggle and (later)
/// close-to-tray both flip this so left-click / "Show/Hide" stay consistent
/// even on backends where querying winit visibility is unreliable (Wayland).
static WINDOW_SHOWN: AtomicBool = AtomicBool::new(true);

/// Last tray-toggle timestamp, for the double-click debounce (see `toggle_window`).
static LAST_TOGGLE: Mutex<Option<Instant>> = Mutex::new(None);
/// Ignore a 2nd tray activation within this window. A tray double-click otherwise
/// fires two `Activate` in milliseconds; on Wayland each toggle destroys/recreates
/// the winit surface AND the wgpu shader underlay, and a render in flight then
/// use-after-frees a `TextureView` (wgpu-core panic). Collapsing the double-click
/// to one toggle avoids the churn.
const TOGGLE_DEBOUNCE_MS: u128 = 400;

/// Toggle the main window: hide if shown, else show + focus.
pub(crate) fn toggle_window(weak: &slint::Weak<AppWindow>) {
    // Debounce: ignore the 2nd click of a tray double-click. Two rapid toggles
    // churn the Wayland surface + wgpu shader underlay and use-after-free a
    // TextureView (wgpu-core panic). The first click always passes.
    {
        let now = Instant::now();
        let mut last = LAST_TOGGLE.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(prev) = *last {
            if now.duration_since(prev).as_millis() < TOGGLE_DEBOUNCE_MS {
                return;
            }
        }
        *last = Some(now);
    }
    if WINDOW_SHOWN.load(Ordering::Relaxed) {
        hide_window(weak);
    } else {
        show_window(weak);
    }
}

/// Show the main window (recreates the Wayland surface) and focus it.
pub(crate) fn show_window(weak: &slint::Weak<AppWindow>) {
    WINDOW_SHOWN.store(true, Ordering::Relaxed);
    let _ = weak.upgrade_in_event_loop(|w| {
        if let Err(e) = w.show() {
            log::error!("[tray] window show failed: {e}");
        }
        // On Wayland the hide destroyed the toplevel and this show recreates
        // it from default attributes — re-apply the persisted size/position/
        // maximized state before the surface maps, or it comes back at the
        // .slint preferred size and the Resized handler persists the loss
        // (#618). Same helper as the startup restore.
        crate::restore_main_window_geometry(&w);
        // Best-effort raise/focus the re-created window (the compositor has
        // the final say on Wayland).
        use i_slint_backend_winit::WinitWindowAccessor;
        w.window().with_winit_window(|win| {
            win.focus_window();
        });
        // Restore the Dock icon when coming back from the menu bar.
        #[cfg(target_os = "macos")]
        macos::set_dock_icon_hidden(false);
    });
}

/// Raise the main window. Activation entry point for MPRIS `Raise` and the
/// single-instance `Present()`. Runs on the event loop; callers may be on any
/// thread.
pub(crate) fn present(weak: &slint::Weak<AppWindow>) {
    let weak = weak.clone();
    let _ = slint::invoke_from_event_loop(move || {
        show_window(&weak);
    });
}

/// Hide the main window to the tray (surface destroyed; the process keeps
/// running, the ksni service stays alive on its own thread).
pub(crate) fn hide_window(weak: &slint::Weak<AppWindow>) {
    WINDOW_SHOWN.store(false, Ordering::Relaxed);
    let _ = weak.upgrade_in_event_loop(|w| {
        // Suppress the dynamic-background shader texture BEFORE the surface is
        // destroyed. It wraps a wgpu texture from THIS surface's instance;
        // drawing it on the NEW instance's first frame after restore is a
        // cross-instance stale-texture use — RefCell panic in debug, SEGFAULT in
        // release. Clearing here (UI thread, no rendering-notifier borrow held)
        // is safe, unlike setting it inside the RenderingTeardown notifier; the
        // 30 fps drain repopulates it on the new instance after restore.
        w.global::<crate::ImmersiveState>()
            .set_shader_texture(slint::Image::default());
        if let Err(e) = w.hide() {
            log::error!("[tray] window hide failed: {e}");
        }
        // Spotify-style opt-in: drop the Dock icon while closed to the menu bar.
        #[cfg(target_os = "macos")]
        if crate::tray_settings::get().mac_hide_dock {
            macos::set_dock_icon_hidden(true);
        }
    });
}

/// Sync the shown-state flag without touching the window — used when Slint
/// itself performs the hide (e.g. an `on_close_requested` → `HideWindow`
/// response) so the next tray toggle knows to show.
pub(crate) fn set_window_shown(shown: bool) {
    WINDOW_SHOWN.store(shown, Ordering::Relaxed);
}
