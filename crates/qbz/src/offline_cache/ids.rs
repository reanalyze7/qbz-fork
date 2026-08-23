//! The session-wide "which track ids are cached" ready-set.

use std::collections::HashSet;
use std::sync::{Mutex as StdMutex, OnceLock};

use qbz_offline_cache::OfflineCacheStatus;

/// Session-wide set of track ids that have a READY offline copy. Seeded from
/// the index.db on login (`load_cached_ids`) and kept in sync as downloads
/// complete / copies are removed. Read at row-build time to seed each row's
/// cache-status (mirrors `fav_cache`), so a cached track shows its check when
/// the view is revisited without re-querying the DB per row.
static CACHED_IDS: OnceLock<StdMutex<HashSet<u64>>> = OnceLock::new();

fn cached_ids() -> &'static StdMutex<HashSet<u64>> {
    CACHED_IDS.get_or_init(|| StdMutex::new(HashSet::new()))
}

/// True if `track_id` (string form, as the row carries it) has a ready offline
/// copy. Used to seed `TrackItem.cache-status` when building track lists.
pub fn is_cached(track_id: &str) -> bool {
    let Ok(id) = track_id.parse::<u64>() else {
        return false;
    };
    cached_ids().lock().map(|s| s.contains(&id)).unwrap_or(false)
}

/// Clone of the ready-set (B8: playlist-snapshot ∩ cached availability
/// checks). Safe from blocking threads — plain mutex, no DB hit.
pub fn cached_ids_set() -> HashSet<u64> {
    cached_ids().lock().map(|s| s.clone()).unwrap_or_default()
}

pub(super) fn mark_cached(track_id: u64, cached: bool) {
    if let Ok(mut set) = cached_ids().lock() {
        if cached {
            set.insert(track_id);
        } else {
            set.remove(&track_id);
        }
    }
}

/// Clear the whole ready-set (used by `clear_all`).
pub(super) fn clear_cached_ids() {
    if let Ok(mut s) = cached_ids().lock() {
        s.clear();
    }
}

/// Seed the ready-set from the active offline cache's index.db. Called once
/// after the offline cache is activated (login / session restore).
pub async fn load_cached_ids() {
    let Some(off) = crate::offline::get().await else {
        return;
    };
    let ids: Vec<u64> = {
        let guard = off.db.lock().await;
        match guard.as_ref() {
            Some(db) => db
                .get_all_tracks()
                .map(|tracks| {
                    tracks
                        .into_iter()
                        .filter(|t| matches!(t.status, OfflineCacheStatus::Ready))
                        .map(|t| t.track_id)
                        .collect()
                })
                .unwrap_or_default(),
            None => Vec::new(),
        }
    };
    if let Ok(mut set) = cached_ids().lock() {
        *set = ids.into_iter().collect();
        log::info!("[qbz-slint] offline: seeded {} cached track ids", set.len());
    }
}
