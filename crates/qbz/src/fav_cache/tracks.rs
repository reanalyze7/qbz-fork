//! Favorite-tracks API.

use std::collections::HashSet;

use super::{FAVORITES, STORE};

/// Replace the cache with a freshly-fetched set and mirror it to the
/// per-user store — full replace, the same semantics as Tauri's
/// `v2_sync_cached_favorite_tracks`. Blocking disk write; call off the
/// UI thread.
pub fn set_all(ids: HashSet<u64>) {
    if let Ok(mut guard) = FAVORITES.write() {
        *guard = ids.clone();
    }
    if let Ok(guard) = STORE.lock() {
        if let Some(store) = guard.as_ref() {
            let disk: Vec<i64> = ids.iter().map(|&id| id as i64).collect();
            if let Err(e) = store.sync_favorite_tracks(&disk) {
                log::warn!("[qbz-slint] favorites cache disk sync failed: {e}");
            }
        }
    }
}

/// True when the given track id (string form) is in the favorite set.
/// Non-numeric ids (local tracks) are never favorites.
pub fn is_favorite(track_id: &str) -> bool {
    let Ok(id) = track_id.parse::<u64>() else {
        return false;
    };
    contains(id)
}

/// Snapshot of the full favorite-track id set. Powers the offline
/// favorites rail (B9): the disk-first seeding makes this correct while
/// offline, right after session activation.
pub fn all() -> HashSet<u64> {
    FAVORITES.read().map(|g| g.clone()).unwrap_or_default()
}

/// True when the given numeric track id is in the favorite set.
pub fn contains(track_id: u64) -> bool {
    FAVORITES
        .read()
        .map(|g| g.contains(&track_id))
        .unwrap_or(false)
}

/// Insert / remove a single id, keeping the cache consistent with an
/// optimistic UI toggle, and mirror the change to the per-user store so
/// hearts survive a restart.
pub fn set(track_id: u64, favorite: bool) {
    if let Ok(mut guard) = FAVORITES.write() {
        if favorite {
            guard.insert(track_id);
        } else {
            guard.remove(&track_id);
        }
    }
    if let Ok(guard) = STORE.lock() {
        if let Some(store) = guard.as_ref() {
            let res = if favorite {
                store.add_favorite_track(track_id as i64)
            } else {
                store.remove_favorite_track(track_id as i64)
            };
            if let Err(e) = res {
                log::warn!("[qbz-slint] favorites cache disk update failed: {e}");
            }
        }
    }
}
