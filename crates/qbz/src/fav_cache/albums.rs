//! Favorite-albums API.

use std::collections::HashSet;

use super::{FAV_ALBUMS, STORE};

/// True when the album catalog id is in the user's favorite-album set.
pub fn is_album_favorite(album_id: &str) -> bool {
    FAV_ALBUMS
        .read()
        .map(|g| g.contains(album_id))
        .unwrap_or(false)
}

/// Replace the favorite-album set with a freshly-fetched id list and mirror
/// it to the per-user store (full replace — Tauri's
/// `v2_sync_cached_favorite_albums`). Blocking disk write; call off the UI
/// thread.
pub fn set_all_albums(ids: HashSet<String>) {
    if let Ok(guard) = STORE.lock() {
        if let Some(store) = guard.as_ref() {
            let disk: Vec<String> = ids.iter().cloned().collect();
            if let Err(e) = store.sync_favorite_albums(&disk) {
                log::warn!("[qbz-slint] favorites cache album sync failed: {e}");
            }
        }
    }
    if let Ok(mut guard) = FAV_ALBUMS.write() {
        *guard = ids;
    }
}

/// Insert / remove a single album id (optimistic toggle) and mirror the
/// change to the per-user store so the heart survives a restart.
pub fn set_album(album_id: &str, favorite: bool) {
    if let Ok(mut guard) = FAV_ALBUMS.write() {
        if favorite {
            guard.insert(album_id.to_string());
        } else {
            guard.remove(album_id);
        }
    }
    if let Ok(guard) = STORE.lock() {
        if let Some(store) = guard.as_ref() {
            let res = if favorite {
                store.add_favorite_album(album_id)
            } else {
                store.remove_favorite_album(album_id)
            };
            if let Err(e) = res {
                log::warn!("[qbz-slint] favorites cache album disk update failed: {e}");
            }
        }
    }
}
