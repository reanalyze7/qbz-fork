use crate::*;

// `WindowEvent::Resized` handling: keep the custom-chrome maximize/restore
// icon honest, and persist the FLOATING window geometry (never the
// maximized/fullscreen footprint) so the next launch restores it. Split
// out of `install_browser_mouse_nav` (part4.rs) to stay under the
// 130-line file cap.
pub(crate) fn handle_window_resized(
    weak: &slint::Weak<AppWindow>,
    slint_window: &i_slint_backend_winit::winit::window::Window,
    size: i_slint_backend_winit::winit::dpi::PhysicalSize<u32>,
) -> EventResult {
                // Keep the custom-chrome maximize/restore icon honest: WM-side
                // maximize (keyboard shortcut, edge-snap, taskbar) never goes
                // through our toggle, so sync the flag from the window state on
                // every resize (cheap — a property set with a change guard
                // inside Slint).
                let (maximized, fullscreen) = if let Some(w) = weak.upgrade() {
                    let maximized = w.window().is_maximized();
                    w.global::<WindowControlActions>().set_is_maximized(maximized);
                    (maximized, w.window().is_fullscreen())
                } else {
                    (false, false)
                };
                let scale = slint_window.scale_factor().max(0.01) as f64;
                let lw = (size.width as f64 / scale) as f32;
                let lh = (size.height as f64 / scale) as f32;
                let mut prefs = crate::ui_prefs::load();
                let mut dirty = prefs.window_maximized != maximized;
                prefs.window_maximized = maximized;
                // window_width/height hold the FLOATING size only: a
                // maximized/fullscreen frame must never overwrite it, or the
                // restore paths (startup + tray/mini re-show, #618) would
                // reproduce the maximized footprint as a floating window.
                if !maximized
                    && !fullscreen
                    && lw >= 940.0 / active_ui_scale()
                    && lh >= 600.0 / active_ui_scale()
                    && ((prefs.window_width - lw).abs() > 0.5
                        || (prefs.window_height - lh).abs() > 0.5)
                {
                    prefs.window_width = lw;
                    prefs.window_height = lh;
                    dirty = true;
                }
                if dirty {
                    crate::ui_prefs::save(&prefs);
                }
                EventResult::Propagate
}
