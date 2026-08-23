//! Multi-select (in-place model edits on the UI thread).

use slint::{ComponentHandle, Model, VecModel};

use crate::{AppWindow, OfflineManagerState, OfflineRow};

fn recount(st: &OfflineManagerState) {
    let model = st.get_rows();
    let mut n = 0;
    for i in 0..model.row_count() {
        if let Some(r) = model.row_data(i) {
            if r.kind == "track" && r.selected {
                n += 1;
            }
        }
    }
    st.set_selected_count(n);
}

/// Flip a single track row's checkbox in place. Plain/Ctrl+Click = single
/// toggle; Shift+Click = additive range from the anchor (always-on selection,
/// so the range is always available). The model interleaves artist-header rows,
/// so the range setter only touches `kind == "track"` rows.
pub fn toggle_select(w: &AppWindow, track_id: &str) {
    let st = w.global::<OfflineManagerState>();
    let model = st.get_rows();
    if let Some(vm) = model.as_any().downcast_ref::<VecModel<OfflineRow>>() {
        let clicked = (0..vm.row_count()).find(|&i| {
            vm.row_data(i)
                .map(|r| r.kind == "track" && r.track_id == track_id)
                .unwrap_or(false)
        });
        if let Some(clicked) = clicked {
            let shift = crate::keybindings::mods().2;
            let anchor = if shift {
                crate::selection::resolve_anchor(
                    crate::selection::SURFACE_OFFLINE,
                    vm,
                    |r| r.track_id.to_string(),
                )
            } else {
                None
            };
            match anchor {
                Some(anchor) => crate::selection::apply_shift_range(vm, anchor, clicked, |r, v| {
                    if r.kind == "track" {
                        r.selected = v;
                    }
                }),
                None => {
                    if let Some(mut r) = vm.row_data(clicked) {
                        r.selected = !r.selected;
                        vm.set_row_data(clicked, r);
                    }
                }
            }
            crate::selection::set_anchor(crate::selection::SURFACE_OFFLINE, clicked, track_id);
        }
    }
    recount(&st);
}

/// Check (or uncheck) every track row.
pub fn set_all_selected(w: &AppWindow, selected: bool) {
    let st = w.global::<OfflineManagerState>();
    let model = st.get_rows();
    if let Some(vm) = model.as_any().downcast_ref::<VecModel<OfflineRow>>() {
        for i in 0..vm.row_count() {
            if let Some(mut r) = vm.row_data(i) {
                if r.kind == "track" && r.selected != selected {
                    r.selected = selected;
                    vm.set_row_data(i, r);
                }
            }
        }
    }
    recount(&st);
}

/// The Qobuz ids of the checked track rows (for the bulk actions).
pub fn selected_track_ids(w: &AppWindow) -> Vec<u64> {
    let model = w.global::<OfflineManagerState>().get_rows();
    let mut ids = Vec::new();
    for i in 0..model.row_count() {
        if let Some(r) = model.row_data(i) {
            if r.kind == "track" && r.selected {
                if let Ok(id) = r.track_id.parse::<u64>() {
                    ids.push(id);
                }
            }
        }
    }
    ids
}
