use crate::*;

/// Poll a future exactly once with a no-op waker. Used for wgpu's native
/// `enumerate_adapters`, which is synchronous under the hood; returns None if it were
/// ever Pending (mirrors Slint's own internal `poll_once`).
pub(crate) fn poll_ready<F: std::future::Future>(fut: F) -> Option<F::Output> {
    use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
    fn clone(_: *const ()) -> RawWaker {
        RawWaker::new(std::ptr::null(), &VTABLE)
    }
    fn noop(_: *const ()) {}
    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, noop, noop, noop);
    // SAFETY: the vtable's clone/wake/drop are all no-ops over a null data pointer, so
    // the Waker is trivially valid and never dereferences the pointer.
    let waker = unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) };
    let mut cx = Context::from_waker(&waker);
    let mut fut = Box::pin(fut);
    match fut.as_mut().poll(&mut cx) {
        Poll::Ready(v) => Some(v),
        Poll::Pending => None,
    }
}

/// Interface-size preset factor active for THIS process run. The persisted
/// pref can change mid-session (Settings dropdown), but the window scale only
/// applies on restart — geometry math must use the factor the window was
/// actually created with, so read this, not the pref.
pub(crate) static ACTIVE_UI_SCALE: std::sync::OnceLock<f32> = std::sync::OnceLock::new();

pub(crate) fn active_ui_scale() -> f32 {
    ACTIVE_UI_SCALE.get().copied().unwrap_or(1.0)
}

/// Re-apply the persisted main-window geometry: LOGICAL size + (best-effort)
/// position, each clamped to the current monitor so a smaller / disconnected
/// display never opens an oversized or stranded window, plus the maximized
/// state. Shared by the startup path and every re-show (tray restore,
/// miniplayer exit): on Wayland a hide DESTROYS the toplevel and the
/// recreated one would otherwise come back at the .slint preferred size
/// (#618). 0 size = never saved → keep the `.slint` preferred size, EXCEPT
/// under a >1 interface-size preset, where even the preferred size may exceed
/// a small monitor once the preset multiplies it — clamp it exactly like a
/// restored one. Monitor sizes are divided by the SLINT scale factor (the
/// effective, preset-baked one); winit's own factor is the raw compositor DPR
/// and under-clamps when a preset is active. The minimum is physically
/// constant across presets (mirrors the `/ UiScale.factor` bindings in
/// app.slint). The monitor query is best-effort (`with_winit_window` returns
/// None before the surface exists — the WM's own clamping is the fallback,
/// and the Resized handler re-saves the result). Position only takes effect
/// on X11/macOS: winit's Wayland `set_outer_position` is a no-op, the
/// compositor places the surface.
pub(crate) fn restore_main_window_geometry(window: &AppWindow) {
    let prefs = crate::ui_prefs::load();
    let ui_scale_factor = active_ui_scale();
    let min_logical_w = 940.0 / ui_scale_factor;
    let min_logical_h = 600.0 / ui_scale_factor;
    // Plausibility gate for the persisted size: at least the scaled minimum,
    // but never stricter than the historical 940x600 (so sizes saved under a
    // previous, less-scaled preset still restore — the .slint mins clamp them
    // up if the current preset needs more).
    let has_saved_size = prefs.window_width >= min_logical_w.min(940.0)
        && prefs.window_height >= min_logical_h.min(600.0);
    if has_saved_size || ui_scale_factor > 1.0 {
        let mut w = if has_saved_size { prefs.window_width } else { 1180.0 };
        let mut h = if has_saved_size { prefs.window_height } else { 760.0 };
        let slint_scale = (window.window().scale_factor() as f64).max(0.01);
        window.window().with_winit_window(|win| {
            if let Some(mon) = win.current_monitor() {
                let avail_w = (mon.size().width as f64 / slint_scale) as f32;
                let avail_h = (mon.size().height as f64 / slint_scale) as f32;
                if avail_w >= min_logical_w {
                    w = w.min(avail_w);
                }
                if avail_h >= min_logical_h {
                    h = h.min(avail_h);
                }
            }
        });
        window.window().set_size(slint::LogicalSize::new(w, h));
    }
    if prefs.window_x != i32::MIN && prefs.window_y != i32::MIN {
        let mut px = prefs.window_x;
        let mut py = prefs.window_y;
        window.window().with_winit_window(|win| {
            if let Some(mon) = win.current_monitor() {
                let m = mon.size();
                let mp = mon.position();
                // Keep the top-left inside the monitor rect, leaving ~100px so a
                // sliver stays grabbable even if the saved spot is near an edge.
                let max_x = (mp.x + m.width as i32 - 100).max(mp.x);
                let max_y = (mp.y + m.height as i32 - 100).max(mp.y);
                px = px.clamp(mp.x, max_x);
                py = py.clamp(mp.y, max_y);
            }
        });
        window
            .window()
            .set_position(slint::PhysicalPosition::new(px, py));
    }
    // Maximized wins over the floating size just applied (which stays the
    // restore target for the eventual un-maximize). Only ever set — never
    // force false, so a freshly created window keeps its natural state.
    if prefs.window_maximized {
        window.window().set_maximized(true);
        window
            .global::<WindowControlActions>()
            .set_is_maximized(true);
    }
}

