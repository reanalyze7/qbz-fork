//! Popular Tracks multi-select (mirrors AlbumState).

use slint::{ComponentHandle, Model};

use crate::{AppWindow, ArtistState};

/// Toggle Popular Tracks multi-select; leaving the mode clears the selection.
pub fn set_multi_select(window: &AppWindow, on: bool) {
    let state = window.global::<ArtistState>();
    state.set_top_tracks_multi_select(on);
    crate::selection::clear_anchor();
    if !on {
        clear_selection(window);
    }
}

/// Recompute the "N selected" count from the Popular Tracks rows.
pub fn recount_selected(window: &AppWindow) {
    let state = window.global::<ArtistState>();
    let model = state.get_top_tracks();
    let count = (0..model.row_count())
        .filter(|&i| model.row_data(i).map(|t| t.selected).unwrap_or(false))
        .count();
    state.set_top_tracks_selected_count(count as i32);
}

/// Select every row, or clear if all are already selected.
pub fn select_all(window: &AppWindow) {
    let model = window.global::<ArtistState>().get_top_tracks();
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

/// Clear the selection (uncheck all), keeping multi-select mode on.
pub fn clear_selection(window: &AppWindow) {
    let model = window.global::<ArtistState>().get_top_tracks();
    for i in 0..model.row_count() {
        if let Some(mut item) = model.row_data(i) {
            if item.selected {
                item.selected = false;
                model.set_row_data(i, item);
            }
        }
    }
    window.global::<ArtistState>().set_top_tracks_selected_count(0);
}

/// Catalog ids of the selected Popular Tracks rows (Qobuz ids only).
pub fn selected_ids(window: &AppWindow) -> Vec<String> {
    let model = window.global::<ArtistState>().get_top_tracks();
    (0..model.row_count())
        .filter_map(|i| model.row_data(i))
        .filter(|t| t.selected)
        .map(|t| t.id.to_string())
        .filter(|s| s.parse::<u64>().is_ok())
        .collect()
}

/// Catalog ids of ALL Popular Tracks rows (for the section "more" menu's
/// all-tracks actions).
pub fn all_top_track_ids(window: &AppWindow) -> Vec<String> {
    let model = window.global::<ArtistState>().get_top_tracks();
    (0..model.row_count())
        .filter_map(|i| model.row_data(i))
        .map(|t| t.id.to_string())
        .filter(|s| s.parse::<u64>().is_ok())
        .collect()
}
