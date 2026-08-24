// SPLIT-EXCEPTION (crates/qbz/src/main.rs refactor): this file holds one
// tightly-sequential Rust function whose internal ordering/control-flow and
// captured-closure state make it unsafe to decompose further without a
// compiler in the loop (no `cargo check` is permitted for this refactor —
// see refactor-plans/crates__qbz__src__main.rs.md). Left whole, over the
// 130-line rule, as the documented rare exception it allows for.
use crate::*;

/// Capture browser mouse buttons before Slint routes them to the topmost
/// TouchArea, and mirror the cursor position into `ShellState` for passive
/// hover chrome that must not install a TouchArea over interactive content.
/// Otherwise cards/sidebar rows can swallow Back/Forward while empty chrome
/// still works, and hover-only scrollbars cannot reliably see card hover.
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
            // Steal Up/Down ONLY while a search dropdown is open, BEFORE the
            // search input's cursor can eat the first press. This is what makes
            // the very first ArrowDown move the selection from the input INTO the
            // dropdown (the input is a single-line TextInput that otherwise
            // consumes the first arrow as a cursor move). Enter/Escape stay with
            // the FocusScope. Two surfaces share this hook: the IMMERSIVE dropdown
            // takes priority when immersive is open + its search is open (it sits
            // on top of everything), otherwise the MAIN header cortinilla.
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

                // (A) Customize-shortcuts editor capture: while recording, the
                // next combo is captured as a binding instead of dispatched.
                let recording = window.global::<KeybindingsState>().get_recording_id();
                if !recording.is_empty() {
                    return crate::keybindings::handle_capture(
                        &window,
                        recording.as_str(),
                        &key_event.logical_key,
                    );
                }

                // (A2) Dead-key compose commits (e.g. us(alt-intl): ' is
                // dead_acute; ' + space = "'", ' + ' = "´", ' + e = "é").
                // winit composes via xkb and delivers the COMMITTED text in
                // `event.text`, but Slint's winit backend maps `logical_key`
                // first and only falls back to `event.text` — so when the
                // second key of the sequence is a named key (Space) or a dead
                // key, the composed character is discarded and an apostrophe
                // can never be typed into any text field. Detect the cases
                // where the composed text differs from what Slint would
                // synthesize and dispatch it ourselves as a synthetic key
                // event, swallowing the raw one. Gated on a focused text
                // input so hotkey behavior outside fields is untouched.
                if window.global::<UiFocusState>().get_text_input_focused() {
                    if let Some(txt) = &key_event.text {
                        let composed_differs = match &key_event.logical_key {
                            // Second key was a printable: winit already folds
                            // the compose result into Key::Character, equal
                            // text means no compose was involved.
                            Key::Character(s) => s.as_str() != txt.as_str(),
                            // ' + space commits the non-combining glyph, but
                            // logical_key stays Named(Space) -> Slint would
                            // insert a plain space.
                            Key::Named(NamedKey::Space) => txt.as_str() != " ",
                            // ' + ' commits the non-combining accent while
                            // logical_key stays Key::Dead -> Slint's fallback
                            // DOES read event.text here, but route it through
                            // the same synthetic path for consistency.
                            Key::Dead(_) => true,
                            _ => false,
                        };
                        if composed_differs && !txt.chars().any(|c| c.is_control()) {
                            let shared: slint::SharedString = txt.as_str().into();
                            window.window().dispatch_event(
                                slint::platform::WindowEvent::KeyPressed { text: shared.clone() },
                            );
                            window.window().dispatch_event(
                                slint::platform::WindowEvent::KeyReleased { text: shared },
                            );
                            return EventResult::PreventDefault;
                        }
                    }
                }

                // (B) Steal Up/Down ONLY while the search dropdown is open,
                // BEFORE the search input's cursor can eat the first press (lets
                // the very first ArrowDown move selection from the input INTO the
                // dropdown).
                let main_cortinilla_open =
                    window.global::<SearchState>().get_cortinilla_open();
                if main_cortinilla_open {
                    let move_selection = |delta: i32| {
                        window
                            .global::<SearchActions>()
                            .invoke_cortinilla_move_selection(delta);
                    };
                    return match &key_event.logical_key {
                        Key::Named(NamedKey::ArrowDown) => {
                            move_selection(1);
                            EventResult::PreventDefault
                        }
                        Key::Named(NamedKey::ArrowUp) => {
                            move_selection(-1);
                            EventResult::PreventDefault
                        }
                        _ => EventResult::Propagate,
                    };
                }

                // (C) Global hotkeys — never while typing in a text field
                // (mirrors the Tauri `isInputTarget` guard).
                if window.global::<UiFocusState>().get_text_input_focused() {
                    return EventResult::Propagate;
                }
                // (C2) Ctrl/Cmd+A = select-all in the active multi-select
                // surface (select-all-ONLY — 1:1 Tauri isSelectAllShortcut).
                // A manual branch because select-all is view+mode contextual,
                // not a rebindable global; falls through to dispatch when no
                // surface is in select mode.
                if keybindings::mods().0 {
                    if let Some(tok) = keybindings::token_from_key(&key_event.logical_key) {
                        if tok.eq_ignore_ascii_case("a") && select_all_active_surface(&window) {
                            return EventResult::PreventDefault;
                        }
                    }
                }
                crate::keybindings::dispatch(&window, &key_event.logical_key)
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
            WindowEvent::Resized(size) => {
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
            WindowEvent::Touch(t) => {
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
            _ => EventResult::Propagate,
        }
    });
}

