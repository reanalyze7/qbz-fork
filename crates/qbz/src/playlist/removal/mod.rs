//! Namespace-split removal (Seam D): resolve selected/single rows to their
//! source-dependent id namespace ahead of a bulk or per-row removal or
//! queue action.

mod queue_tracks;
mod split;

pub use queue_tracks::selected_queue_tracks;
pub use split::split_for_removal;

use slint::{ComponentHandle, Model};

use crate::{AppWindow, PlaylistState};

use crate::playlist::view_state::FULL_ITEMS;

/// A row reference for removal: the display id + the row's source — the
/// id namespace is source-dependent (catalog id / library row id). Built
/// from the selection (bulk) or a single row (the per-row "Remove from
/// playlist" menu entry rides this same seam when it lands).
pub struct SelectedRow {
    pub id: String,
    pub source: String,
}

/// The currently-selected rows with their sources.
pub fn selected_rows(window: &AppWindow) -> Vec<SelectedRow> {
    let model = window.global::<PlaylistState>().get_tracks();
    (0..model.row_count())
        .filter_map(|i| model.row_data(i))
        .filter(|t| t.selected)
        .map(|t| SelectedRow {
            id: t.id.to_string(),
            source: t.source.to_string(),
        })
        .collect()
}

/// A single row (id + source) by display id — the per-row "Remove from
/// playlist" menu entry rides the same namespace-split seam as the bulk
/// selection, with a one-row set. UI thread.
pub fn row_for_id(id: &str) -> Option<SelectedRow> {
    FULL_ITEMS.with(|cell| {
        cell.borrow()
            .iter()
            .find(|item| item.id.as_str() == id)
            .map(|item| SelectedRow {
                id: item.id.to_string(),
                source: item.source.to_string(),
            })
    })
}
