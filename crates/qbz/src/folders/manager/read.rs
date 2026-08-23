//! Playlist Manager read-side: full folder records + per-playlist
//! settings/stats maps.

use std::collections::HashMap;

use crate::library_db;

/// Full folder record for the Playlist Manager (icon + color + hidden).
#[derive(Clone, Default)]
pub struct FolderFull {
    pub id: String,
    pub name: String,
    pub icon_type: String,
    pub icon_preset: String,
    pub icon_color: String,
    pub custom_image_path: Option<String>,
    pub is_hidden: bool,
}

/// Per-playlist local settings the manager merges onto the remote list.
#[derive(Clone, Default)]
pub struct PlaylistSettingsLite {
    pub hidden: bool,
    pub is_favorite: bool,
    pub position: i32,
    pub folder_id: Option<String>,
}

/// All folders with their full icon/color records, ordered by position.
pub fn load_folders_full() -> Vec<FolderFull> {
    library_db::with_db(|db| db.get_all_playlist_folders())
        .unwrap_or_default()
        .into_iter()
        .map(|f| FolderFull {
            id: f.id,
            name: f.name,
            icon_type: f.icon_type,
            icon_preset: f.icon_preset,
            icon_color: f.icon_color,
            custom_image_path: f.custom_image_path,
            is_hidden: f.is_hidden,
        })
        .collect()
}

/// playlist id -> its local settings (hidden/favorite/position/folder).
pub fn playlist_settings_map() -> HashMap<u64, PlaylistSettingsLite> {
    library_db::with_db(|db| db.get_all_playlist_settings())
        .unwrap_or_default()
        .into_iter()
        .map(|s| {
            (
                s.qobuz_playlist_id,
                PlaylistSettingsLite {
                    hidden: s.hidden,
                    is_favorite: s.is_favorite,
                    position: s.position,
                    folder_id: s.folder_id,
                },
            )
        })
        .collect()
}

/// playlist id -> play count (for the "Play Count" sort + the list badge).
pub fn playlist_play_counts() -> HashMap<u64, u32> {
    library_db::with_db(|db| db.get_all_playlist_stats())
        .unwrap_or_default()
        .into_iter()
        .map(|s| (s.qobuz_playlist_id, s.play_count))
        .collect()
}

/// playlist id -> local (non-Qobuz) track count.
pub fn playlist_local_counts() -> HashMap<u64, u32> {
    library_db::with_db(|db| db.get_all_playlist_local_track_counts()).unwrap_or_default()
}
