use crate::*;

// `WindowEvent::Touch` handling: log the first native touch event
// (diagnostic for compositor mouse-emulation), and mirror the touch point
// into ShellState so hover-revealed chrome reacts under native touch.
// Split out of `install_browser_mouse_nav` (part4.rs) to stay under the
// 130-line file cap.
pub(crate) fn handle_window_touch(
    weak: &slint::Weak<AppWindow>,
    slint_window: &slint::Window,
    t: &i_slint_backend_winit::winit::event::Touch,
) -> EventResult {
                // Diagnostic: confirm the compositor actually delivers NATIVE
                // touch. RaspiOS/labwc defaults to touchscreen mouse-emulation,
                // which never sends wl_touch to clients — taps survive as
                // emulated clicks but the motion stream Slint's Flickable needs
                // for swipe/drag never arrives. If this line never logs after
                // tapping on the Pi, mouse-emulation is on (fix compositor-side).
                static TOUCH_SEEN: std::sync::Once = std::sync::Once::new();
                TOUCH_SEEN.call_once(|| {
                    log::info!(
                        "[input] native touch delivered by compositor (id={}, phase={:?})",
                        t.id,
                        t.phase
                    );
                });
                // Mirror the touch point to ShellState so hover-revealed chrome
                // (scrollbars, row actions) reacts under native touch, where
                // there is no CursorMoved stream.
                if matches!(t.phase, TouchPhase::Started | TouchPhase::Moved) {
                    if let Some(window) = weak.upgrade() {
                        let position =
                            t.location.to_logical::<f64>(slint_window.scale_factor() as f64);
                        let state = window.global::<ShellState>();
                        state.set_pointer_x(position.x as f32);
                        state.set_pointer_y(position.y as f32);
                        state.set_pointer_in_window(true);
                    }
                }
                EventResult::Propagate
}
