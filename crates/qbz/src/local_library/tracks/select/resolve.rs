//! Resolve the current selection / a row id to `LocalTrack`.

use slint::{ComponentHandle, Model};

use crate::{AppWindow, LocalLibraryState};

use crate::local_library::tracks::load::tracks_current;

/// The selected rows resolved to `LocalTrack`, in DISPLAY order (iterate the
/// visible model, look each selected id up in `TRACKS_CURRENT`). Deviates from
/// favorites' load-order resolution so the queue matches what the user sees.
pub fn selected_local_tracks(window: &AppWindow) -> Vec<qbz_library::LocalTrack> {
    let model = window.global::<LocalLibraryState>().get_tracks_visible();
    let cache = tracks_current();
    let mut out = Vec::new();
    for i in 0..model.row_count() {
        if let Some(item) = model.row_data(i) {
            if item.selected {
                let id = item.id.to_string();
                if let Some(t) = cache.iter().find(|t| t.id.to_string() == id) {
                    out.push(t.clone());
                }
            }
        }
    }
    out
}

/// Resolve a single row id (display) to its `LocalTrack` from the cache.
pub fn local_track_by_id(id: &str) -> Option<qbz_library::LocalTrack> {
    tracks_current().iter().find(|t| t.id.to_string() == id).cloned()
}
