//! Callback wiring (KeybindingsActions) — called once at startup.

use slint::ComponentHandle;

use crate::{AppWindow, KeybindingsActions, KeybindingsState};

use super::bindings::{reset_all, reset_one};
use super::model::refresh;

pub fn wire(window: &AppWindow) {
    let actions = window.global::<KeybindingsActions>();

    let weak = window.as_weak();
    actions.on_start_record(move |id| {
        if let Some(w) = weak.upgrade() {
            let s = w.global::<KeybindingsState>();
            s.set_recording_id(id);
            s.set_pending_display("".into());
            s.set_conflict_label("".into());
        }
    });

    let weak = window.as_weak();
    actions.on_cancel_record(move || {
        if let Some(w) = weak.upgrade() {
            let s = w.global::<KeybindingsState>();
            s.set_recording_id("".into());
            s.set_pending_display("".into());
            s.set_conflict_label("".into());
        }
    });

    let weak = window.as_weak();
    actions.on_reset_one(move |id| {
        reset_one(id.as_str());
        if let Some(w) = weak.upgrade() {
            refresh(&w);
        }
    });

    let weak = window.as_weak();
    actions.on_reset_all(move || {
        reset_all();
        if let Some(w) = weak.upgrade() {
            refresh(&w);
        }
    });

    refresh(window);
}
