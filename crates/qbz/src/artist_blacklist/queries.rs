//! Read-only snapshots + listings for the manager view and filter code.

use std::collections::HashSet;

use qbz_app::settings::artist_blacklist::{BlacklistedAlbum, BlacklistedArtist};

use super::lifecycle::with_service;

/// Snapshot of the full blacklisted-id set, for `qbz_core::search_all`-style
/// filtering. Empty when no session is bound. Derived from `get_all` so it
/// reflects the persisted rows (ignores the enabled flag — callers gate on
/// [`super::is_enabled`] separately).
pub fn ids_snapshot() -> HashSet<u64> {
    with_service(HashSet::new(), |s| {
        s.get_all()
            .map(|list| list.into_iter().map(|a| a.artist_id).collect())
            .unwrap_or_default()
    })
}

/// All blacklisted artists (name-sorted), for the manager view. Empty on no
/// session or query error.
pub fn get_all() -> Vec<BlacklistedArtist> {
    with_service(Vec::new(), |s| s.get_all().unwrap_or_default())
}

/// Count of blacklisted artists (ignores the enabled flag). `0` when no session
/// is bound.
pub fn count() -> usize {
    with_service(0, |s| s.count())
}

/// Snapshot of the full blocked-album-id set, for `qbz_core` album/track
/// filtering. Empty when no session is bound. Reflects persisted rows (ignores
/// the enabled flag — callers gate on [`super::is_enabled`] separately).
pub fn album_ids_snapshot() -> HashSet<String> {
    with_service(HashSet::new(), |s| {
        s.get_all_albums()
            .map(|list| list.into_iter().map(|a| a.album_id).collect())
            .unwrap_or_default()
    })
}

/// All blocked albums (title-sorted), for the manager view. Empty on no session
/// or query error.
pub fn get_all_albums() -> Vec<BlacklistedAlbum> {
    with_service(Vec::new(), |s| s.get_all_albums().unwrap_or_default())
}

/// Count of blocked albums (ignores the enabled flag). `0` when no session is
/// bound.
pub fn album_count() -> usize {
    with_service(0, |s| s.album_count())
}
