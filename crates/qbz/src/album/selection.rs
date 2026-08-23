//! Multi-select (track selection) on the album track list.

use qbz_models::Track;
use slint::{ComponentHandle, Model};

use crate::{AlbumState, AppWindow};

use super::PLAY_TRACKS;

/// Toggle multi-select mode on the album track list. Leaving the mode
/// clears any current selection.
pub fn set_multi_select(window: &AppWindow, on: bool) {
    let state = window.global::<AlbumState>();
    state.set_multi_select(on);
    // Reset the Shift-range anchor whenever the mode changes (fresh session on
    // enter, no stale anchor on leave).
    crate::selection::clear_anchor();
    if !on {
        clear_selection(window);
    }
}

/// Recompute the "N selected" count from the track rows.
pub fn recount_selected(window: &AppWindow) {
    let state = window.global::<AlbumState>();
    let model = state.get_tracks();
    let count = (0..model.row_count())
        .filter(|&i| model.row_data(i).map(|t| t.selected).unwrap_or(false))
        .count();
    state.set_selected_count(count as i32);
}

/// Select every row, or clear if all are already selected (the toggle the
/// "Select all" bulk button drives — same semantics as the favorites bar).
pub fn select_all(window: &AppWindow) {
    let model = window.global::<AlbumState>().get_tracks();
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
    let model = window.global::<AlbumState>().get_tracks();
    for i in 0..model.row_count() {
        if let Some(mut item) = model.row_data(i) {
            if item.selected {
                item.selected = false;
                model.set_row_data(i, item);
            }
        }
    }
    window.global::<AlbumState>().set_selected_count(0);
}

/// The catalog ids of the currently selected rows (for add-to-playlist /
/// add-to-favorites — Qobuz catalog ids only).
pub fn selected_ids(window: &AppWindow) -> Vec<String> {
    let model = window.global::<AlbumState>().get_tracks();
    (0..model.row_count())
        .filter_map(|i| model.row_data(i))
        .filter(|t| t.selected)
        .map(|t| t.id.to_string())
        .filter(|s| s.parse::<u64>().is_ok())
        .collect()
}

/// The full catalog Track objects for the selected rows (for enqueue /
/// cache), resolved from the stashed raw album tracks by id.
pub fn selected_play_tracks(window: &AppWindow) -> Vec<Track> {
    let ids: std::collections::HashSet<String> = selected_ids(window).into_iter().collect();
    PLAY_TRACKS.with(|cell| {
        cell.borrow()
            .iter()
            .filter(|t| ids.contains(&t.id.to_string()))
            .cloned()
            .collect()
    })
}

/// The full catalog Track objects for one disc of the open album (for the
/// per-disc "Disc N" header menu), resolved from the stashed raw album tracks.
/// `disc` matches `media_number` (defaulting to 1, exactly as `map_track`
/// stamps `TrackData.disc`), so it lines up with the rendered "Disc N" header.
/// Preserves the delivered (disc-then-track) order.
pub fn disc_play_tracks(disc: i32) -> Vec<Track> {
    PLAY_TRACKS.with(|cell| {
        cell.borrow()
            .iter()
            .filter(|t| t.media_number.unwrap_or(1) as i32 == disc)
            .cloned()
            .collect()
    })
}
