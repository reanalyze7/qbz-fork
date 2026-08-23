//! Generic span-fill helpers over a `VecModel`.

use slint::{Model, VecModel};

/// Additive Shift-range over a `VecModel`: set selected = true for every row in
/// the inclusive index span `[min(anchor,clicked), max(anchor,clicked)]`. Never
/// deselects (1:1 with `applyShiftRange`). Generic over the row type via the
/// `set_selected` setter closure (e.g. `|t, v| t.selected = v`).
pub fn apply_shift_range<T: Clone + 'static>(
    model: &VecModel<T>,
    anchor: usize,
    clicked: usize,
    set_selected: impl Fn(&mut T, bool),
) {
    let lo = anchor.min(clicked);
    let hi = anchor.max(clicked);
    let n = model.row_count();
    for i in lo..=hi {
        if i < n {
            if let Some(mut item) = model.row_data(i) {
                set_selected(&mut item, true);
                model.set_row_data(i, item);
            }
        }
    }
}

/// Select-all-ONLY over a `VecModel`: set selected = true for every row (never
/// toggles to clear — 1:1 with Tauri's `isSelectAllShortcut`, which only ever
/// selects all; the toggling all-or-none lives on the bulk bar's button). The
/// caller recounts afterwards.
pub fn select_all<T: Clone + 'static>(model: &VecModel<T>, set_selected: impl Fn(&mut T, bool)) {
    for i in 0..model.row_count() {
        if let Some(mut item) = model.row_data(i) {
            set_selected(&mut item, true);
            model.set_row_data(i, item);
        }
    }
}
