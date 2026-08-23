//! Rendered-album-id enumeration + the per-model patch helper shared by
//! every select operation below.

use slint::{ComponentHandle, Model, ModelRc};

use crate::{AlbumCardItem, AppWindow, LocalLibraryState};

/// Rendered album ids in display order (flat `albums-visible`, or the
/// concatenation of the grouped sections when grouping is on).
pub(crate) fn rendered_album_ids(window: &AppWindow) -> Vec<String> {
    let s = window.global::<LocalLibraryState>();
    let grouped = s.get_albums_grouped();
    if grouped.row_count() > 0 {
        let mut out = Vec::new();
        for gi in 0..grouped.row_count() {
            if let Some(sec) = grouped.row_data(gi) {
                for i in 0..sec.albums.row_count() {
                    if let Some(a) = sec.albums.row_data(i) {
                        out.push(a.id.to_string());
                    }
                }
            }
        }
        out
    } else {
        let m = s.get_albums_visible();
        (0..m.row_count())
            .filter_map(|i| m.row_data(i))
            .map(|a| a.id.to_string())
            .collect()
    }
}

/// Apply `f` to each rendered album model (flat `albums-visible` + every grouped
/// section). The source `albums` set is intentionally NOT touched — a re-sort /
/// re-filter rebuilds the rendered set from it and resets the selection, which
/// matches Tauri's anchor-reset-on-rebuild behavior.
pub(crate) fn for_each_album_model(window: &AppWindow, mut f: impl FnMut(&ModelRc<AlbumCardItem>)) {
    let s = window.global::<LocalLibraryState>();
    f(&s.get_albums_visible());
    let grouped = s.get_albums_grouped();
    for gi in 0..grouped.row_count() {
        if let Some(sec) = grouped.row_data(gi) {
            f(&sec.albums);
        }
    }
}

/// Set `selected = value` on every album row whose id is in `ids`.
pub(crate) fn set_albums_selected(
    window: &AppWindow,
    ids: &std::collections::HashSet<String>,
    value: bool,
) {
    for_each_album_model(window, |m| {
        for i in 0..m.row_count() {
            if let Some(mut it) = m.row_data(i) {
                if ids.contains(it.id.as_str()) && it.selected != value {
                    it.selected = value;
                    m.set_row_data(i, it);
                }
            }
        }
    });
}

/// Current selected state of one album id (read from the rendered set).
pub(crate) fn album_is_selected(window: &AppWindow, id: &str) -> bool {
    let mut found = false;
    for_each_album_model(window, |m| {
        if found {
            return;
        }
        for i in 0..m.row_count() {
            if let Some(a) = m.row_data(i) {
                if a.id.as_str() == id {
                    found = a.selected;
                    return;
                }
            }
        }
    });
    found
}

/// Recount distinct selected albums into `albums-selected-count`.
pub fn recount_albums_selected(window: &AppWindow) {
    let mut seen = std::collections::HashSet::new();
    for_each_album_model(window, |m| {
        for i in 0..m.row_count() {
            if let Some(a) = m.row_data(i) {
                if a.selected {
                    seen.insert(a.id.to_string());
                }
            }
        }
    });
    window
        .global::<LocalLibraryState>()
        .set_albums_selected_count(seen.len() as i32);
}
