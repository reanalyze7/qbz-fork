//! Followed (favorite) artists API.

use std::collections::HashSet;

use super::{FAV_ARTISTS, STORE};

/// True when the artist id is in the user's followed-artist set.
pub fn is_artist_favorite(artist_id: u64) -> bool {
    FAV_ARTISTS
        .read()
        .map(|g| g.contains(&artist_id))
        .unwrap_or(false)
}

/// Snapshot of the full followed-artist id set — the reco paint filter's
/// "already following" exclusion input (mirrors `all()` for tracks).
pub fn all_artists() -> HashSet<u64> {
    FAV_ARTISTS.read().map(|g| g.clone()).unwrap_or_default()
}

/// Replace the followed-artist set with a freshly-fetched id list and mirror
/// it to the per-user store (full replace — same lifecycle as the album set).
/// Blocking disk write; call off the UI thread.
pub fn set_all_artists(ids: HashSet<u64>) {
    if let Ok(guard) = STORE.lock() {
        if let Some(store) = guard.as_ref() {
            let disk: Vec<i64> = ids.iter().map(|&id| id as i64).collect();
            if let Err(e) = store.sync_favorite_artists(&disk) {
                log::warn!("[qbz-slint] favorites cache artist sync failed: {e}");
            }
        }
    }
    if let Ok(mut guard) = FAV_ARTISTS.write() {
        *guard = ids;
    }
}

/// Insert / remove a single artist id (optimistic follow toggle) and mirror
/// the change to the per-user store so the follow survives a restart.
pub fn set_artist(artist_id: u64, favorite: bool) {
    if let Ok(mut guard) = FAV_ARTISTS.write() {
        if favorite {
            guard.insert(artist_id);
        } else {
            guard.remove(&artist_id);
        }
    }
    if let Ok(guard) = STORE.lock() {
        if let Some(store) = guard.as_ref() {
            let res = if favorite {
                store.add_favorite_artist(artist_id as i64)
            } else {
                store.remove_favorite_artist(artist_id as i64)
            };
            if let Err(e) = res {
                log::warn!("[qbz-slint] favorites cache artist disk update failed: {e}");
            }
        }
    }
}
