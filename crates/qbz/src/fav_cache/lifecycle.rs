//! Per-user store lifecycle: disk-first seeding of all three favorite sets.

use std::collections::HashSet;
use std::path::Path;

use qbz_app::settings::favorites_cache::FavoritesCacheStore;

use super::{FAVORITES, FAV_ALBUMS, FAV_ARTISTS, STORE};

/// Bind the per-user store and seed the in-memory set from disk (works
/// offline). Called on every session activation — login, restore, and
/// offline entry — next to `offline_mode::init_for_user`. Best-effort:
/// failures are logged and leave the set empty (hearts render unfavorited,
/// never block entry).
pub fn init_for_user(base_dir: &Path) {
    let store = match FavoritesCacheStore::new_at(base_dir) {
        Ok(store) => store,
        Err(e) => {
            log::error!("[qbz-slint] favorites cache store open failed: {e}");
            return;
        }
    };
    match store.get_favorite_track_ids() {
        Ok(ids) => {
            let set: HashSet<u64> = ids
                .into_iter()
                .filter_map(|id| u64::try_from(id).ok())
                .collect();
            log::info!(
                "[qbz-slint] favorites cache: {} track ids seeded from disk",
                set.len()
            );
            if let Ok(mut guard) = FAVORITES.write() {
                *guard = set;
            }
        }
        Err(e) => log::warn!("[qbz-slint] favorites cache disk seed failed: {e}"),
    }
    match store.get_favorite_album_ids() {
        Ok(ids) => {
            let set: HashSet<String> = ids.into_iter().collect();
            log::info!(
                "[qbz-slint] favorites cache: {} album ids seeded from disk",
                set.len()
            );
            if let Ok(mut guard) = FAV_ALBUMS.write() {
                *guard = set;
            }
        }
        Err(e) => log::warn!("[qbz-slint] favorites cache album disk seed failed: {e}"),
    }
    match store.get_favorite_artist_ids() {
        Ok(ids) => {
            let set: HashSet<u64> = ids
                .into_iter()
                .filter_map(|id| u64::try_from(id).ok())
                .collect();
            log::info!(
                "[qbz-slint] favorites cache: {} artist ids seeded from disk",
                set.len()
            );
            if let Ok(mut guard) = FAV_ARTISTS.write() {
                *guard = set;
            }
        }
        Err(e) => log::warn!("[qbz-slint] favorites cache artist disk seed failed: {e}"),
    }
    if let Ok(mut guard) = STORE.lock() {
        *guard = Some(store);
    }
}

/// Drop the per-user store and the in-memory set on logout.
pub fn teardown() {
    if let Ok(mut guard) = STORE.lock() {
        *guard = None;
    }
    if let Ok(mut guard) = FAVORITES.write() {
        guard.clear();
    }
    if let Ok(mut guard) = FAV_ALBUMS.write() {
        guard.clear();
    }
    if let Ok(mut guard) = FAV_ARTISTS.write() {
        guard.clear();
    }
}
