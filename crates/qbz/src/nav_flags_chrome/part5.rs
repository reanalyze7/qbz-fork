use crate::*;

/// Wire the immersive header's native window-control cluster
/// (WindowControlActions) to the MAIN AppWindow's winit handle. The AppWindow
/// uses native OS decorations, so the only seam to move/min/max/fullscreen it
/// is the winit backend — reached via `WinitWindowAccessor::with_winit_window`,
/// exactly as miniplayer.rs does for the mini window. `with_winit_window`
/// returns `Option<T>` (None when not on the winit backend / window gone), so
/// every handler degrades gracefully. Read-then-set state (maximize/fullscreen)
/// returns the new flag OUT of the closure, then pushes it onto the global
/// AFTER the borrow ends.
pub(crate) fn wire_window_controls(window: &AppWindow) {
    let actions = window.global::<WindowControlActions>();

    // Window state (minimize / maximize / fullscreen) is driven through SLINT's
    // own Window API, NOT a direct winit call. The winit adapter reconciles the
    // window from Slint's own properties on every realization
    // (winitwindowadapter.rs ~1328 toggles winit to match `properties.is_fullscreen()`),
    // so a direct `winit.set_fullscreen(..)` gets UNDONE on the next frame — the
    // same "use Slint's native mechanism, don't fight winit" lesson as the
    // miniplayer's decorations fix. `slint::Window::{set_minimized,set_maximized,
    // set_fullscreen}` (api.rs:576/586/596) write the property the adapter reads.

    // Minimize — ALWAYS a normal WM minimize (owner decision 2026-07-03).
    // The old Tauri-parity branch routed this through minimize-to-tray, which
    // made the button visually identical to close-to-tray (window vanishes);
    // the titlebar button must minimize, period. The minimize-to-tray setting
    // keeps its Settings row but no longer affects this button.
    {
        let weak = window.as_weak();
        actions.on_minimize(move || {
            if let Some(w) = weak.upgrade() {
                log::info!("[qbz-slint] minimize (titlebar): WM minimize");
                w.window().set_minimized(true);
            }
        });
    }

    // Maximize / Restore toggle.
    {
        let weak = window.as_weak();
        actions.on_toggle_maximize(move || {
            if let Some(w) = weak.upgrade() {
                let m = !w.window().is_maximized();
                w.window().set_maximized(m);
                w.global::<WindowControlActions>().set_is_maximized(m);
            }
        });
    }

    // Fullscreen toggle (true fullscreen hides the native titlebar — the
    // genuinely useful immersive control). MUST go through slint::Window so the
    // realization reconciliation keeps it instead of reverting it.
    {
        let weak = window.as_weak();
        actions.on_toggle_fullscreen(move || {
            if let Some(w) = weak.upgrade() {
                let fs = !w.window().is_fullscreen();
                w.window().set_fullscreen(fs);
                w.global::<WindowControlActions>().set_is_fullscreen(fs);
            }
        });
    }

    // Close app — reuse the AppWindow's existing close-app choreography
    // (close-to-tray vs quit lives in `window.on_close_app`, main.rs ~13558;
    // miniplayer.rs calls the same `invoke_close_app`).
    {
        let weak = window.as_weak();
        actions.on_close_app(move || {
            if let Some(w) = weak.upgrade() {
                w.invoke_close_app();
            }
        });
    }

    // Drag-move — start a window-move drag (same idiom as miniplayer start_drag).
    {
        let weak = window.as_weak();
        actions.on_drag_move(move || {
            if let Some(w) = weak.upgrade() {
                w.window().with_winit_window(|win| {
                    let _ = win.drag_window();
                });
            }
        });
    }
}

/// Whether the CURRENT content view is one the AppShell swaps for the
/// OfflinePlaceholder while offline. KEEP IN SYNC with `qobuz-view-blocked`
/// in `AppShell.slint`. The playlist view blocks only when it is neither a
/// LOCAL playlist nor the offline sidecar rendering of a mixed one (D11.a).
/// UI thread only (reads the globals).
pub(crate) fn is_offline_blocked_view(window: &AppWindow) -> bool {
    match window.global::<NavState>().get_view() {
        ContentView::Home
        | ContentView::DiscoverBrowse
        | ContentView::PlaylistBrowse
        | ContentView::Search
        | ContentView::Favorites
        | ContentView::Album
        | ContentView::Artist
        | ContentView::Musician
        | ContentView::Label
        | ContentView::LabelReleases
        | ContentView::Location
        | ContentView::Mix => true,
        ContentView::Playlist => {
            let ps = window.global::<PlaylistState>();
            !ps.get_is_local() && !ps.get_offline_subset()
        }
        _ => false,
    }
}

