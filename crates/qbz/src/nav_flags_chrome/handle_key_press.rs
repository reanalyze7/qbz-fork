use crate::*;

// `WindowEvent::KeyboardInput` (pressed) handling: customize-shortcuts
// capture, dead-key compose commits, the search-dropdown Up/Down steal,
// global hotkeys (guarded off text fields), Ctrl/Cmd+A select-all, and the
// keybindings dispatch fallback. Split out of `install_browser_mouse_nav`
// (part4.rs) to stay under the 130-line file cap.
pub(crate) fn handle_key_press(
    window: &AppWindow,
    key_event: &i_slint_backend_winit::winit::event::KeyEvent,
) -> EventResult {
                // (A) Customize-shortcuts editor capture: while recording, the
                // next combo is captured as a binding instead of dispatched.
                let recording = window.global::<KeybindingsState>().get_recording_id();
                if !recording.is_empty() {
                    return crate::keybindings::handle_capture(
                        window,
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
                        if tok.eq_ignore_ascii_case("a") && select_all_active_surface(window) {
                            return EventResult::PreventDefault;
                        }
                    }
                }
                crate::keybindings::dispatch(window, &key_event.logical_key)
}
