//! Modifier tracking (UI thread only).

use std::cell::Cell;

thread_local! {
    static MODS: Cell<(bool, bool, bool)> = const { Cell::new((false, false, false)) };
}

/// Record the current modifier state from a winit `ModifiersChanged` event.
/// `ctrl` already folds in Meta/Super (mirrors the TS `ctrlKey || metaKey`).
pub fn set_mods(ctrl: bool, alt: bool, shift: bool) {
    MODS.with(|m| m.set((ctrl, alt, shift)));
}

/// Current modifier state `(ctrl, alt, shift)` as last reported by winit's
/// `ModifiersChanged`. `ctrl` already folds in Meta/Super. Read by the keyboard
/// dispatch AND by the multi-select toggle arm (to decide Shift-range vs single
/// toggle at click time — see `selection`).
pub fn mods() -> (bool, bool, bool) {
    MODS.with(|m| m.get())
}
