//! Enter/leave multi-select, per-row toggle (with shift-range), select-all,
//! clear.

use slint::{ComponentHandle, Model};

use crate::{AppWindow, LocalLibraryState, TrackItem};

/// Enter/leave multi-select; leaving clears the selection.
pub fn set_tracks_multi_select(window: &AppWindow, on: bool) {
    window.global::<LocalLibraryState>().set_tracks_multi_select(on);
    crate::selection::clear_anchor();
    if !on {
        clear_tracks_selection(window);
    }
}

/// Recount selected visible rows into `tracks-selected-count`.
pub fn recount_tracks_selected(window: &AppWindow) {
    let s = window.global::<LocalLibraryState>();
    let model = s.get_tracks_visible();
    let count = (0..model.row_count())
        .filter(|&i| model.row_data(i).map(|t| t.selected).unwrap_or(false))
        .count();
    s.set_tracks_selected_count(count as i32);
}

/// Toggle one row's selection (by id) in the visible model. Plain/Ctrl+Click =
/// single toggle; Shift+Click = additive range from the anchor (1:1 with the
/// central track arm — LocalLibrary routes its own toggle, not that arm).
pub fn toggle_track_select(window: &AppWindow, id: &str) {
    let model = window.global::<LocalLibraryState>().get_tracks_visible();
    if let Some(vm) = model.as_any().downcast_ref::<slint::VecModel<TrackItem>>() {
        let clicked = (0..vm.row_count())
            .find(|&i| vm.row_data(i).map(|t| t.id.as_str() == id).unwrap_or(false));
        if let Some(clicked) = clicked {
            let shift = crate::keybindings::mods().2;
            let anchor = if shift {
                crate::selection::resolve_anchor(
                    crate::selection::SURFACE_LOCAL_TRACKS,
                    vm,
                    |t| t.id.to_string(),
                )
            } else {
                None
            };
            match anchor {
                Some(anchor) => crate::selection::apply_shift_range(
                    vm,
                    anchor,
                    clicked,
                    |t, v| t.selected = v,
                ),
                None => {
                    if let Some(mut item) = vm.row_data(clicked) {
                        item.selected = !item.selected;
                        vm.set_row_data(clicked, item);
                    }
                }
            }
            crate::selection::set_anchor(crate::selection::SURFACE_LOCAL_TRACKS, clicked, id);
        }
    }
    recount_tracks_selected(window);
}

/// Select-all toggle: select every visible row, or clear if all selected.
pub fn select_all_tracks(window: &AppWindow) {
    let model = window.global::<LocalLibraryState>().get_tracks_visible();
    let total = model.row_count();
    let selected = (0..total)
        .filter(|&i| model.row_data(i).map(|t| t.selected).unwrap_or(false))
        .count();
    let target = selected != total;
    for i in 0..total {
        if let Some(mut item) = model.row_data(i) {
            if item.selected != target {
                item.selected = target;
                model.set_row_data(i, item);
            }
        }
    }
    recount_tracks_selected(window);
}

/// Deselect every visible row (multi-select mode stays on).
pub fn clear_tracks_selection(window: &AppWindow) {
    let s = window.global::<LocalLibraryState>();
    let model = s.get_tracks_visible();
    for i in 0..model.row_count() {
        if let Some(mut item) = model.row_data(i) {
            if item.selected {
                item.selected = false;
                model.set_row_data(i, item);
            }
        }
    }
    s.set_tracks_selected_count(0);
}
