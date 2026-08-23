//! Cache mutation + rebuild triggers.

use crate::AppWindow;

use super::rebuild::rebuild;
use super::{SidebarData, CACHE, EXPANDED, NAME_DESC};

/// Store the freshly-loaded data and render it.
pub fn apply(window: &AppWindow, data: SidebarData) {
    if let Ok(mut cache) = CACHE.lock() {
        *cache = data;
    }
    rebuild(window);
}

/// Optimistically patch a playlist's displayed NAME after a successful
/// rename (Qobuz numeric id or "local:<uuid>"), then re-render from the
/// patched cache. The edit-playlist handler still triggers a full
/// `load_sidebar_playlists` afterwards to reconcile — this patch exists
/// because the reload alone does not reliably show the new name right away
/// (Qobuz playlist/list read-after-write lag): the row must reflect the
/// edit the moment the modal closes.
pub fn rename_entry(window: &AppWindow, id: &str, name: &str) {
    if let Ok(mut cache) = CACHE.lock() {
        if let Ok(numeric) = id.parse::<u64>() {
            if let Some(p) = cache.playlists.iter_mut().find(|p| p.id == numeric) {
                p.name = name.to_string();
            }
        }
        if let Some(p) = cache.local_playlists.iter_mut().find(|p| p.id == id) {
            p.name = name.to_string();
        }
    }
    // Keep the session name/desc cache in sync too — the edit modal and the
    // offline name synthesis both prefill from it.
    if let (Ok(numeric), Ok(mut nd)) = (id.parse::<u64>(), NAME_DESC.lock()) {
        if let Some(entry) = nd.get_mut(&numeric) {
            entry.0 = name.to_string();
        }
    }
    rebuild(window);
}

/// Toggle a folder's expanded state, then re-render from cache.
pub fn toggle_folder(window: &AppWindow, folder_id: &str) {
    if let Ok(mut exp) = EXPANDED.lock() {
        if !exp.remove(folder_id) {
            exp.insert(folder_id.to_string());
        }
    }
    rebuild(window);
}

/// Optimistically move a playlist in the cache (folder_id "" = root)
/// and re-render. The DB write happens separately.
pub fn move_playlist_local(window: &AppWindow, playlist_id: u64, folder_id: &str) {
    if let Ok(mut cache) = CACHE.lock() {
        if folder_id.is_empty() {
            cache.folder_map.remove(&playlist_id);
        } else {
            cache.folder_map.insert(playlist_id, folder_id.to_string());
        }
    }
    rebuild(window);
}

/// Optimistically move a LOCAL playlist (`local:<uuid>` id) into a folder
/// (`folder_id` "" = root) in the cache, then re-render. The DB write happens
/// separately. The local twin of `move_playlist_local`.
pub fn move_local_playlist_local(window: &AppWindow, id: &str, folder_id: &str) {
    if let Ok(mut cache) = CACHE.lock() {
        if let Some(p) = cache.local_playlists.iter_mut().find(|p| p.id == id) {
            p.folder_id = if folder_id.is_empty() {
                None
            } else {
                Some(folder_id.to_string())
            };
        }
    }
    rebuild(window);
}

// (removed `contains` — playlist ownership/follow is now decided by the Qobuz
// owner id vs the current user + get_user_playlists membership, not by sidebar
// presence, which is only a CONSEQUENCE of following. See main.rs playlist load.)
