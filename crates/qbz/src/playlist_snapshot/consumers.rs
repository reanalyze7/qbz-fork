//! Blocking consumer fns — call from `spawn_blocking`.

use std::collections::{HashMap, HashSet};

use qbz_library::qobuz_playlist_snapshot as repo;

/// All snapshot headers: playlist id -> (name, total Qobuz track count at
/// snapshot time). Blocking — call from `spawn_blocking`.
pub fn headers_blocking() -> HashMap<u64, (String, Option<u32>)> {
    crate::library_db::with_db(|db| Ok(db.with_connection(repo::all_headers)))
        .and_then(|r| r.ok())
        .map(|headers| {
            headers
                .into_iter()
                .map(|h| (h.qobuz_playlist_id, (h.name, h.track_count)))
                .collect()
        })
        .unwrap_or_default()
}

/// Snapshot name of one playlist. Blocking.
pub fn name_blocking(playlist_id: u64) -> Option<String> {
    crate::library_db::with_db(|db| {
        Ok(db.with_connection(|conn| repo::get_header(conn, playlist_id)))
    })
    .and_then(|r| r.ok())
    .flatten()
    .map(|h| h.name)
}

/// B8 availability: the Qobuz playlists whose snapshot membership intersects
/// the offline-cache ready set — i.e. they have at least one track playable
/// offline right now. Empty past the subscription grace window (D4: the
/// cache may not serve full tracks, so nothing is playable). Blocking.
pub fn available_offline_blocking() -> HashSet<u64> {
    if !crate::offline_mode::offline_playback_allowed() {
        return HashSet::new();
    }
    let cached = crate::offline_cache::cached_ids_set();
    if cached.is_empty() {
        return HashSet::new();
    }
    crate::library_db::with_db(|db| Ok(db.with_connection(repo::all_track_ids)))
        .and_then(|r| r.ok())
        .map(|memberships| {
            memberships
                .into_iter()
                .filter(|(_, ids)| ids.iter().any(|id| cached.contains(id)))
                .map(|(id, _)| id)
                .collect()
        })
        .unwrap_or_default()
}

/// B8 detail: one playlist's snapshot track ids that are PLAYABLE offline
/// (cached + within the grace window), in snapshot position order. Blocking.
pub fn playable_track_ids_blocking(playlist_id: u64) -> Vec<u64> {
    if !crate::offline_mode::offline_playback_allowed() {
        return Vec::new();
    }
    let cached = crate::offline_cache::cached_ids_set();
    if cached.is_empty() {
        return Vec::new();
    }
    crate::library_db::with_db(|db| {
        Ok(db.with_connection(|conn| repo::track_ids(conn, playlist_id)))
    })
    .and_then(|r| r.ok())
    .map(|ids| ids.into_iter().filter(|id| cached.contains(id)).collect())
    .unwrap_or_default()
}
