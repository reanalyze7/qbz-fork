//! Multi-select for the landing page's Popular Tracks list.

use qbz_models::Track;
use slint::{ComponentHandle, Model};

use super::top_tracks_for_play;
use crate::{AppWindow, LabelState};

/// Toggle Popular Tracks multi-select; leaving clears the selection + anchor.
pub fn set_multi_select(window: &AppWindow, on: bool) {
    window.global::<LabelState>().set_multi_select(on);
    crate::selection::clear_anchor();
    if !on {
        clear_selection(window);
    }
}

/// Recompute the "N selected" count from the Popular Tracks rows.
pub fn recount_selected(window: &AppWindow) {
    let state = window.global::<LabelState>();
    let model = state.get_top_tracks();
    let count = (0..model.row_count())
        .filter(|&i| model.row_data(i).map(|t| t.selected).unwrap_or(false))
        .count();
    state.set_selected_count(count as i32);
}

/// Select every row, or clear if all are already selected (the bulk bar's
/// "Select all" toggle — Ctrl+A goes through `selection::select_all`).
pub fn select_all(window: &AppWindow) {
    let model = window.global::<LabelState>().get_top_tracks();
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
    let model = window.global::<LabelState>().get_top_tracks();
    for i in 0..model.row_count() {
        if let Some(mut item) = model.row_data(i) {
            if item.selected {
                item.selected = false;
                model.set_row_data(i, item);
            }
        }
    }
    window.global::<LabelState>().set_selected_count(0);
}

/// Catalog ids of the selected rows (for add-to-playlist — Qobuz ids only).
pub fn selected_ids(window: &AppWindow) -> Vec<String> {
    let model = window.global::<LabelState>().get_top_tracks();
    (0..model.row_count())
        .filter_map(|i| model.row_data(i))
        .filter(|t| t.selected)
        .map(|t| t.id.to_string())
        .filter(|s| s.parse::<u64>().is_ok())
        .collect()
}

/// Full `Track` objects for the selected rows (for enqueue), resolved from the
/// cached top tracks in DISPLAY order.
pub fn selected_play_tracks(window: &AppWindow) -> Vec<Track> {
    let model = window.global::<LabelState>().get_top_tracks();
    let cur = top_tracks_for_play();
    (0..model.row_count())
        .filter_map(|i| model.row_data(i))
        .filter(|t| t.selected)
        .filter_map(|t| {
            let id = t.id.to_string();
            cur.iter().find(|c| c.id.to_string() == id).cloned()
        })
        .collect()
}
