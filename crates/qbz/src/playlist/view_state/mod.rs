//! Search/sort rendering core: the `FULL_ITEMS`/`QUERY`/`SORT` thread-locals
//! and the `refresh_view` derivation everything else (custom order, artwork,
//! multi-select) calls into.

mod sort;

pub(super) use sort::refresh_view;

use crate::{AppWindow, PlaylistState, TrackItem};
use slint::ComponentHandle;

thread_local! {
    /// The full, original-order row list — the canonical source the
    /// search + sort derive the visible list from. UI thread only.
    pub(super) static FULL_ITEMS: std::cell::RefCell<Vec<TrackItem>> = std::cell::RefCell::new(Vec::new());
    /// Active in-page search query.
    pub(super) static QUERY: std::cell::RefCell<String> = std::cell::RefCell::new(String::new());
    /// Active sort: (field, ascending). field "default" = playlist order.
    pub(super) static SORT: std::cell::RefCell<(String, bool)> =
        std::cell::RefCell::new(("default".to_string(), true));
}

/// The `(track_id, is_local)` custom-order key of a row, derived from the
/// row's source the way Tauri derives it from `isLocal` (§1.3):
/// - Qobuz rows -> `(catalog id, false)`.
/// - Local sidecar rows -> `(library row id, true)` — same value Tauri
///   stores (`local_tracks.id`, `is_local=1`).
/// - Rows without a stable numeric id (`file:`/`broken:` fallbacks) ->
///   None: excluded from the order, sorted to the end.
pub(super) fn custom_key(item: &TrackItem) -> Option<(u64, bool)> {
    let is_local = item.source.as_str() == "local";
    item.id.parse::<u64>().ok().map(|id| (id, is_local))
}

/// Resolve a row's artwork into BOTH the stable FULL_ITEMS list (so a
/// later sort/filter keeps it — they rebuild from FULL_ITEMS) and the
/// visible model row, matched by id (the displayed order may differ
/// from FULL_ITEMS after a sort). Called by the artwork pipeline.
pub fn set_track_artwork(window: &AppWindow, full_index: usize, image: slint::Image) {
    use slint::Model;
    let id = FULL_ITEMS.with(|c| {
        let mut b = c.borrow_mut();
        b.get_mut(full_index).map(|it| {
            it.artwork = image.clone();
            it.id.clone()
        })
    });
    let Some(id) = id else { return };
    let model = window.global::<PlaylistState>().get_tracks();
    for i in 0..model.row_count() {
        if let Some(mut it) = model.row_data(i) {
            if it.id == id {
                it.artwork = image;
                model.set_row_data(i, it);
                break;
            }
        }
    }
}

/// Update the search query and re-render.
pub fn filter_tracks(window: &AppWindow, query: &str) {
    QUERY.with(|q| *q.borrow_mut() = query.to_string());
    refresh_view(window);
}

/// Set the sort field. Re-selecting the active field toggles asc/desc;
/// "default" restores playlist order. Mirrors Tauri's behaviour.
pub fn set_sort(window: &AppWindow, field: &str) {
    SORT.with(|s| {
        let mut cur = s.borrow_mut();
        if field == "default" || field == "custom" {
            *cur = (field.to_string(), true);
        } else if cur.0 == field {
            cur.1 = !cur.1;
        } else {
            // "added" starts newest-first (desc) — v1.x parity; the other
            // fields start ascending.
            *cur = (field.to_string(), field != "added");
        }
    });
    let (field, asc) = SORT.with(|s| s.borrow().clone());
    let state = window.global::<PlaylistState>();
    state.set_sort_field(field.into());
    state.set_sort_asc(asc);
    refresh_view(window);
}
