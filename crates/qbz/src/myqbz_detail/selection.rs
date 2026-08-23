//! Multi-select edit mode.

use qbz_models::mixtape::MixtapeCollectionItem;
use slint::{ComponentHandle, Model};

use super::FULL_ITEMS;
use crate::{AppWindow, MyQbzDetailState};

/// Toggle multi-select edit mode. Leaving clears any selection.
pub fn toggle_select_mode(window: &AppWindow) {
    let state = window.global::<MyQbzDetailState>();
    let on = !state.get_select_mode();
    if !on {
        let model = state.get_items();
        for i in 0..model.row_count() {
            if let Some(mut it) = model.row_data(i) {
                if it.selected {
                    it.selected = false;
                    model.set_row_data(i, it);
                }
            }
        }
        state.set_selected_count(0);
    }
    state.set_select_mode(on);
}

/// Toggle one row's selection by position. Recounts the selection.
pub fn toggle_item_select(window: &AppWindow, position: i32) {
    let state = window.global::<MyQbzDetailState>();
    let model = state.get_items();
    for i in 0..model.row_count() {
        if let Some(mut it) = model.row_data(i) {
            if it.position == position {
                it.selected = !it.selected;
                model.set_row_data(i, it);
                break;
            }
        }
    }
    let count = (0..model.row_count())
        .filter(|&i| model.row_data(i).map(|it| it.selected).unwrap_or(false))
        .count() as i32;
    state.set_selected_count(count);
}

/// The set of currently-selected row positions (select-mode), read off the
/// rendered item model. UI thread.
pub fn selected_positions(window: &AppWindow) -> Vec<i32> {
    let model = window.global::<MyQbzDetailState>().get_items();
    (0..model.row_count())
        .filter_map(|i| model.row_data(i))
        .filter(|it| it.selected)
        .map(|it| it.position)
        .collect()
}

/// The full `MixtapeCollectionItem`s (with year / track_count) for the
/// currently-selected positions, in ascending position order. Sourced from
/// `FULL_ITEMS` (the slint `MixtapeDetailItem` carries only display text, not
/// the numeric year/track_count the add payload needs). UI thread.
pub fn selected_full_items(window: &AppWindow) -> Vec<MixtapeCollectionItem> {
    let mut positions = selected_positions(window);
    positions.sort_unstable();
    FULL_ITEMS.with(|cell| {
        let items = cell.borrow();
        positions
            .iter()
            .filter_map(|p| items.iter().find(|it| it.position == *p).cloned())
            .collect()
    })
}

/// Clear the current selection (uncheck every row + zero the count), staying in
/// select-mode. Used after a bulk action completes. UI thread.
pub fn clear_selection(window: &AppWindow) {
    let state = window.global::<MyQbzDetailState>();
    let model = state.get_items();
    for i in 0..model.row_count() {
        if let Some(mut it) = model.row_data(i) {
            if it.selected {
                it.selected = false;
                model.set_row_data(i, it);
            }
        }
    }
    state.set_selected_count(0);
}
