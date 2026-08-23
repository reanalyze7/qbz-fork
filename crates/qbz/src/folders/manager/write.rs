//! Playlist Manager write-side: folder create/update, favorite/hidden
//! flags, reorder.

use crate::library_db;

use super::read::FolderFull;

/// Create a folder with icon preset + color (manager create path).
pub fn create_folder_full(name: &str, icon_preset: &str, icon_color: &str) -> Option<FolderFull> {
    let preset = Some(icon_preset);
    let color = if icon_color.is_empty() {
        None
    } else {
        Some(icon_color)
    };
    library_db::with_db(|db| db.create_playlist_folder(name, Some("preset"), preset, color)).map(
        |f| FolderFull {
            id: f.id,
            name: f.name,
            icon_type: f.icon_type,
            icon_preset: f.icon_preset,
            icon_color: f.icon_color,
            custom_image_path: f.custom_image_path,
            is_hidden: f.is_hidden,
        },
    )
}

/// Update a folder (name, icon preset/type, color, custom image, hidden).
/// `custom_image_path` is `Some(Some(p))` to set, `Some(None)` to clear,
/// `None` to leave unchanged (mirrors the DB signature).
#[allow(clippy::too_many_arguments)]
pub fn update_folder_full(
    id: &str,
    name: &str,
    icon_type: &str,
    icon_preset: &str,
    icon_color: &str,
    custom_image_path: Option<Option<&str>>,
    is_hidden: bool,
) {
    let color = if icon_color.is_empty() {
        None
    } else {
        Some(icon_color)
    };
    library_db::with_db(|db| {
        db.update_playlist_folder(
            id,
            Some(name),
            Some(icon_type),
            Some(icon_preset),
            color,
            custom_image_path,
            Some(is_hidden),
        )
    });
}

/// Set a playlist's favorite flag.
pub fn set_favorite(playlist_id: u64, favorite: bool) {
    library_db::with_db(|db| db.set_playlist_favorite(playlist_id, favorite));
}

/// Set a playlist's hidden flag.
pub fn set_hidden(playlist_id: u64, hidden: bool) {
    library_db::with_db(|db| db.set_playlist_hidden(playlist_id, hidden));
}

/// Set a folder's hidden flag (leaves all other fields unchanged).
pub fn set_folder_hidden(id: &str, hidden: bool) {
    library_db::with_db(|db| {
        db.update_playlist_folder(id, None, None, None, None, None, Some(hidden))
    });
}

/// Persist a custom playlist order (custom-sort positions).
pub fn reorder_playlists(playlist_ids: &[u64]) {
    library_db::with_db(|db| db.reorder_playlists(playlist_ids));
}
