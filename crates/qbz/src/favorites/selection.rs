//! Tracks multi-select (mirrors playlist.rs). Selection lives on
//! TrackItem.selected in the rendered `tracks-visible` model (which shares
//! `tracks` when no search filter is active).

use qbz_models::Track;
use slint::{ComponentHandle, Model};

use crate::favorites::random::FAV_CURRENT;
use crate::{AppWindow, FavoritesState};

/// Enter/leave multi-select mode; leaving clears the selection.
pub fn set_multi_select(window: &AppWindow, on: bool) {
    window.global::<FavoritesState>().set_tracks_multi_select(on);
    crate::selection::clear_anchor();
    if !on {
        clear_selection(window);
    }
}

/// Recount selected rows into `tracks-selected-count`.
pub fn recount_selected(window: &AppWindow) {
    let state = window.global::<FavoritesState>();
    let model = state.get_tracks_visible();
    let count = (0..model.row_count())
        .filter(|&i| model.row_data(i).map(|t| t.selected).unwrap_or(false))
        .count();
    state.set_tracks_selected_count(count as i32);
}

/// Select-all toggle: select every visible row, or clear if all selected.
pub fn select_all(window: &AppWindow) {
    let model = window.global::<FavoritesState>().get_tracks_visible();
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
    recount_selected(window);
}

/// Deselect every visible row.
pub fn clear_selection(window: &AppWindow) {
    let state = window.global::<FavoritesState>();
    let model = state.get_tracks_visible();
    for i in 0..model.row_count() {
        if let Some(mut item) = model.row_data(i) {
            if item.selected {
                item.selected = false;
                model.set_row_data(i, item);
            }
        }
    }
    state.set_tracks_selected_count(0);
}

/// The ids of the currently-selected visible rows.
pub fn selected_ids(window: &AppWindow) -> Vec<String> {
    let model = window.global::<FavoritesState>().get_tracks_visible();
    (0..model.row_count())
        .filter_map(|i| model.row_data(i))
        .filter(|t| t.selected)
        .map(|t| t.id.to_string())
        .collect()
}

/// The selected favorite tracks as full Track objects (for bulk enqueue),
/// in favorites order.
pub fn selected_tracks(window: &AppWindow) -> Vec<Track> {
    let ids = selected_ids(window);
    if ids.is_empty() {
        return Vec::new();
    }
    FAV_CURRENT
        .lock()
        .map(|c| {
            c.iter()
                .filter(|t| ids.contains(&t.id.to_string()))
                .cloned()
                .collect()
        })
        .unwrap_or_default()
}
