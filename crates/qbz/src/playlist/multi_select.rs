//! Multi-select edit mode: enter/leave, select-all, and the selected-count
//! readout.

use slint::{ComponentHandle, Model};

use crate::{AppWindow, PlaylistState};

/// Recount selected rows into PlaylistState.selected-count.
pub fn recount_selected(window: &AppWindow) {
    let model = window.global::<PlaylistState>().get_tracks();
    let count = (0..model.row_count())
        .filter(|&i| model.row_data(i).map(|t| t.selected).unwrap_or(false))
        .count() as i32;
    window.global::<PlaylistState>().set_selected_count(count);
}

/// Enter/leave edit mode. Leaving clears any selection.
pub fn set_multi_select(window: &AppWindow, on: bool) {
    if !on {
        clear_selection(window);
    }
    crate::selection::clear_anchor();
    window.global::<PlaylistState>().set_multi_select_mode(on);
}

/// Clear the selection WITHOUT leaving multi-select mode — the bulk
/// queueing actions keep the mode active (LocalLibrary bulk precedent).
pub fn clear_selection(window: &AppWindow) {
    let state = window.global::<PlaylistState>();
    let model = state.get_tracks();
    for i in 0..model.row_count() {
        if let Some(mut item) = model.row_data(i) {
            if item.selected {
                item.selected = false;
                model.set_row_data(i, item);
            }
        }
    }
    state.set_selected_count(0);
}

/// Toggle select-all: select every row, or clear if all are selected.
pub fn select_all(window: &AppWindow) {
    let model = window.global::<PlaylistState>().get_tracks();
    let total = model.row_count();
    let selected = (0..total)
        .filter(|&i| model.row_data(i).map(|t| t.selected).unwrap_or(false))
        .count();
    let target = selected != total; // if not all selected -> select all
    for i in 0..total {
        if let Some(mut item) = model.row_data(i) {
            if item.selected != target {
                item.selected = target;
                model.set_row_data(i, item);
            }
        }
    }
    recount_selected(window);
}
