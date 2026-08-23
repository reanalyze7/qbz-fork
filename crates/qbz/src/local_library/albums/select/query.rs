//! Resolve the current album selection to ids / tracks (for the bulk bar).

use crate::AppWindow;

use super::basics::{album_is_selected, rendered_album_ids};

/// The selected album ids (group keys), in rendered order.
pub fn selected_album_ids(window: &AppWindow) -> Vec<String> {
    let ids = rendered_album_ids(window);
    ids.into_iter()
        .filter(|id| album_is_selected(window, id))
        .collect()
}

/// Resolve the selected albums to their `LocalTrack`s (BLOCKING — DB; call from
/// `spawn_blocking`). Each album's tracks are concatenated in rendered order.
pub fn selected_albums_tracks_blocking(album_keys: &[String]) -> Vec<qbz_library::LocalTrack> {
    let mut out = Vec::new();
    for key in album_keys {
        out.extend(crate::local_library::shared::fetch_album_tracks_blocking(key));
    }
    out
}
