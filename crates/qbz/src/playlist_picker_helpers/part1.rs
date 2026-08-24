use crate::*;

/// Look up a playlist's display name from the picker state model by id
/// (the picker only carries names UI-side in `PlaylistPickItem`). Used for
/// the "Added N tracks to <name>" success toast. Falls back to an empty
/// string when the id is not found.
pub(crate) fn picker_playlist_name(w: &AppWindow, id: &str) -> String {
    use slint::Model;
    let model = w.global::<PlaylistPickerState>().get_playlists();
    for i in 0..model.row_count() {
        if let Some(item) = model.row_data(i) {
            if item.id == id {
                return item.name.to_string();
            }
        }
    }
    String::new()
}

/// Success toast for a playlist add ("Added N tracks to <playlist>"). Hops
/// onto the event loop, so it is safe to call from a worker task. An empty
/// `name` degrades to "Added N tracks". The count is the number actually
/// written.
pub(crate) fn toast_added_tracks(weak: &slint::Weak<AppWindow>, count: usize, name: String) {
    if count == 0 {
        return;
    }
    let msg = if name.is_empty() {
        format!("Added {count} tracks")
    } else {
        format!("Added {count} tracks to {name}")
    };
    crate::toast::success_weak(weak, msg);
}

/// Success toast for a playlist removal ("Removed N tracks from
/// <playlist>"), mirrors `toast_added_tracks`.
pub(crate) fn toast_removed_tracks(weak: &slint::Weak<AppWindow>, count: usize, name: String) {
    if count == 0 {
        return;
    }
    let msg = if name.is_empty() {
        format!("Removed {count} tracks")
    } else {
        format!("Removed {count} tracks from {name}")
    };
    crate::toast::success_weak(weak, msg);
}

