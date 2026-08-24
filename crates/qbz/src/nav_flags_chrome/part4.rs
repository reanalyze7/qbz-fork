use crate::*;

/// Capture browser mouse buttons before Slint routes them to the topmost
/// TouchArea, and mirror the cursor position into `ShellState` for passive
/// hover chrome that must not install a TouchArea over interactive content.
/// Otherwise cards/sidebar rows can swallow Back/Forward while empty chrome
/// still works, and hover-only scrollbars cannot reliably see card hover.
///
/// The `KeyboardInput` / `Resized` / `Touch` arms are split into
/// `handle_key_press`, `handle_window_resized`, `handle_window_touch`
/// (this dir's `handle_*.rs`) to stay under the 130-line file cap.
pub(crate) fn install_browser_mouse_nav(window: &AppWindow) {
    let weak = window.as_weak();
    window.window().on_winit_window_event(move |slint_window, event| {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                let Some(window) = weak.upgrade() else {
                    return EventResult::Propagate;
                };
                let position = position.to_logical::<f64>(slint_window.scale_factor() as f64);
                let state = window.global::<ShellState>();
                state.set_pointer_x(position.x as f32);
                state.set_pointer_y(position.y as f32);
                state.set_pointer_in_window(true);
                return EventResult::Propagate;
            }
            WindowEvent::CursorLeft { .. } => {
                if let Some(window) = weak.upgrade() {
                    let state = window.global::<ShellState>();
                    state.set_pointer_in_window(false);
                    state.set_pointer_x(-1.0);
                    state.set_pointer_y(-1.0);
                }
                return EventResult::Propagate;
            }
            // A deliberate close is also liveness (a crash never emits one) —
            // without this, a launch-then-close-from-the-taskbar inside the
            // fallback window would falsely degrade the renderer next start.
            WindowEvent::CloseRequested => {
                disarm_renderer_sentinel_on_liveness("close request");
                return EventResult::Propagate;
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button,
                ..
            } => {
                // A real click = the app reached a usable state; the startup
                // renderer sentinel can stand down (see its doc).
                disarm_renderer_sentinel_on_liveness("mouse input");
                let Some(window) = weak.upgrade() else {
                    return EventResult::Propagate;
                };

                match button {
                    MouseButton::Back => {
                        window.global::<NavState>().invoke_request_back();
                        EventResult::PreventDefault
                    }
                    MouseButton::Forward => {
                        window.global::<NavState>().invoke_request_forward();
                        EventResult::PreventDefault
                    }
                    _ => EventResult::Propagate,
                }
            }
            // Track modifier state for the keybindings dispatcher (winit
            // delivers modifiers separately from key presses).
            WindowEvent::ModifiersChanged(modifiers) => {
                let m = modifiers.state();
                crate::keybindings::set_mods(
                    m.control_key() || m.super_key(),
                    m.alt_key(),
                    m.shift_key(),
                );
                EventResult::Propagate
            }
            WindowEvent::KeyboardInput { event: key_event, .. }
                if key_event.state == ElementState::Pressed =>
            {
                // Key press = usable app; stand the startup sentinel down.
                disarm_renderer_sentinel_on_liveness("key input");
                let Some(window) = weak.upgrade() else {
                    return EventResult::Propagate;
                };
                handle_key_press(&window, key_event)
            }
            // Persist main-window geometry so the next launch restores it
            // (mirrors miniplayer.rs). The startup restore clamps to the
            // monitor; here we just record what the WM settled on. A change
            // guard avoids redundant writes on the many no-op events the WM
            // emits. The app minimum is 940x600 PHYSICAL-equivalent — the
            // .slint mins divide the interface-size preset out, so the
            // LOGICAL minimum scales with it (XL windows sit well below 940
            // logical). Ignore smaller frames (minimize reports 0x0,
            // mid-transition frames undershoot).
            WindowEvent::Resized(size) => handle_window_resized(&weak, slint_window, *size),
            WindowEvent::Moved(pos) => {
                // Same floating-only rule as the size: the maximized origin
                // (often 0,0) must not overwrite the floating position.
                let floating = weak
                    .upgrade()
                    .map(|w| !w.window().is_maximized() && !w.window().is_fullscreen())
                    .unwrap_or(false);
                if floating {
                    let mut prefs = crate::ui_prefs::load();
                    if prefs.window_x != pos.x || prefs.window_y != pos.y {
                        prefs.window_x = pos.x;
                        prefs.window_y = pos.y;
                        crate::ui_prefs::save(&prefs);
                    }
                }
                EventResult::Propagate
            }
            WindowEvent::Touch(t) => handle_window_touch(&weak, slint_window, t),
            _ => EventResult::Propagate,
        }
    });
}
