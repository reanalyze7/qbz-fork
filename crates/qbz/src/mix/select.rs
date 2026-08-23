//! Multi-select (track selection) over the mix track list.

use qbz_models::Track;
use slint::{ComponentHandle, Model};

use crate::{AppWindow, MixState};

use super::state::current_tracks;

/// Toggle multi-select mode; leaving the mode clears the selection. Drops the
/// Shift-range anchor on either transition.
pub fn set_multi_select(window: &AppWindow, on: bool) {
    window.global::<MixState>().set_multi_select(on);
    crate::selection::clear_anchor();
    if !on {
        clear_selection(window);
    }
}

/// Recompute the "N selected" count from the track rows.
pub fn recount_selected(window: &AppWindow) {
    let state = window.global::<MixState>();
    let model = state.get_tracks();
    let count = (0..model.row_count())
        .filter(|&i| model.row_data(i).map(|t| t.selected).unwrap_or(false))
        .count();
    state.set_selected_count(count as i32);
}

/// Select every row, or clear if all are already selected (the toggle the
/// "Select all" bulk button drives — same semantics as the album/favorites bar;
/// Ctrl+A goes through `selection::select_all`, which only ever selects).
pub fn select_all(window: &AppWindow) {
    let model = window.global::<MixState>().get_tracks();
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
    let model = window.global::<MixState>().get_tracks();
    for i in 0..model.row_count() {
        if let Some(mut item) = model.row_data(i) {
            if item.selected {
                item.selected = false;
                model.set_row_data(i, item);
            }
        }
    }
    window.global::<MixState>().set_selected_count(0);
}

/// The catalog ids of the selected rows (for add-to-playlist — Qobuz ids only).
pub fn selected_ids(window: &AppWindow) -> Vec<String> {
    let model = window.global::<MixState>().get_tracks();
    (0..model.row_count())
        .filter_map(|i| model.row_data(i))
        .filter(|t| t.selected)
        .map(|t| t.id.to_string())
        .filter(|s| s.parse::<u64>().is_ok())
        .collect()
}

/// The full `Track` objects for the selected rows (for enqueue), resolved from
/// the cached mix tracks in DISPLAY order.
pub fn selected_play_tracks(window: &AppWindow) -> Vec<Track> {
    let model = window.global::<MixState>().get_tracks();
    let cur = current_tracks();
    (0..model.row_count())
        .filter_map(|i| model.row_data(i))
        .filter(|t| t.selected)
        .filter_map(|t| {
            let id = t.id.to_string();
            cur.iter().find(|c| c.id.to_string() == id).cloned()
        })
        .collect()
}
