//! The lightweight id+name folder API the sidebar controller uses.

use std::collections::HashMap;

use crate::library_db;

#[derive(Clone)]
pub struct FolderInfo {
    pub id: String,
    pub name: String,
}

/// All folders, ordered by their stored position. The sidebar now uses
/// `load_folders_full` (it needs the hidden flag to exclude hidden
/// folders); kept as a lightweight id+name helper for other callers.
#[allow(dead_code)]
pub fn load_folders() -> Vec<FolderInfo> {
    library_db::with_db(|db| db.get_all_playlist_folders())
        .unwrap_or_default()
        .into_iter()
        .map(|f| FolderInfo {
            id: f.id,
            name: f.name,
        })
        .collect()
}

/// playlist id -> folder id, for grouping playlists under folders.
pub fn playlist_folder_map() -> HashMap<u64, String> {
    library_db::with_db(|db| db.get_all_playlist_settings())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|s| s.folder_id.map(|fid| (s.qobuz_playlist_id, fid)))
        .collect()
}

/// playlist id -> custom-sort position, for the sidebar "Custom" sort.
pub fn playlist_positions() -> HashMap<u64, i32> {
    library_db::with_db(|db| db.get_all_playlist_settings())
        .unwrap_or_default()
        .into_iter()
        .map(|s| (s.qobuz_playlist_id, s.position))
        .collect()
}

pub fn create_folder(name: &str) -> Option<FolderInfo> {
    library_db::with_db(|db| db.create_playlist_folder(name, None, None, None)).map(|f| {
        FolderInfo {
            id: f.id,
            name: f.name,
        }
    })
}

pub fn delete_folder(id: &str) {
    library_db::with_db(|db| db.delete_playlist_folder(id));
    // The shared `playlist_folders` FK is `ON DELETE SET NULL`, but the app's
    // connections keep the foreign_keys pragma off, so null the LOCAL members'
    // folder_id explicitly (the Qobuz side is handled by delete_playlist_folder).
    library_db::with_db(|db| {
        Ok(db.with_connection(|conn| qbz_library::local_playlists::clear_folder(conn, id)))
    });
}

/// Move a playlist into `folder_id`, or to root when None.
pub fn move_playlist(playlist_id: u64, folder_id: Option<&str>) {
    library_db::with_db(|db| db.move_playlist_to_folder(playlist_id, folder_id));
}

/// Move a LOCAL playlist (`local:<uuid>` id) into `folder_id`, or to root when
/// None. Persists to the `local_playlists.folder_id` column (shared folders).
pub fn move_local_playlist(id: &str, folder_id: Option<&str>) {
    library_db::with_db(|db| {
        Ok(db.with_connection(|conn| {
            qbz_library::local_playlists::move_to_folder(conn, id, folder_id)
        }))
    });
}
