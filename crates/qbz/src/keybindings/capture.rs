//! Capture (the customize editor's "press a key" widget).

use i_slint_backend_winit::{
    winit::keyboard::{Key, NamedKey},
    EventResult,
};
use slint::ComponentHandle;

use crate::{AppWindow, KeybindingsState};

use super::bindings::{active_bindings, conflicting_action, set_binding};
use super::grammar::{format_display, shortcut_from_parts, token_from_key};
use super::mods::mods;
use super::model::refresh;

/// Handle a keypress while the customize editor is recording a binding for
/// `action_id`. Always consumes the event.
pub fn handle_capture(window: &AppWindow, action_id: &str, key: &Key) -> EventResult {
    let state = window.global::<KeybindingsState>();

    // Escape cancels (does not bind — Escape stays the ui.escape default).
    if matches!(key, Key::Named(NamedKey::Escape)) {
        state.set_recording_id("".into());
        state.set_pending_display("".into());
        state.set_conflict_label("".into());
        return EventResult::PreventDefault;
    }

    let (ctrl, alt, shift) = mods();
    let Some(token) = token_from_key(key) else {
        // Bare modifier / unrepresentable — ignore, keep recording.
        return EventResult::PreventDefault;
    };
    let Some(shortcut) = shortcut_from_parts(ctrl, alt, shift, &token) else {
        return EventResult::PreventDefault;
    };

    state.set_pending_display(format_display(&shortcut).into());
    let bindings = active_bindings();
    if let Some(conflict) = conflicting_action(&shortcut, action_id, &bindings) {
        state.set_conflict_label(qbz_i18n::t(conflict.label_en).into());
        // Leave recording on so the user can pick a different combo.
    } else {
        set_binding(action_id, &shortcut);
        refresh(window);
        state.set_recording_id("".into());
        state.set_pending_display("".into());
        state.set_conflict_label("".into());
    }
    EventResult::PreventDefault
}
