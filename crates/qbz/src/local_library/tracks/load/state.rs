//! Generation guard, the loaded-rows cache, and the raw page fetch.

use std::sync::atomic::AtomicU64;
use std::sync::{LazyLock, Mutex};

use crate::local_library::shared::exclude_network_folders_now;

pub(crate) const TRACKS_PAGE: u64 = 200;

pub(crate) static TRACKS_GEN: AtomicU64 = AtomicU64::new(0);

/// The loaded `LocalTrack` rows backing `tracks`, kept in lockstep with the
/// paged model in apply/append. The selection-source for bulk queue/play-next/
/// add-to-playlist (resolves ids -> LocalTrack with no DB round-trip).
static TRACKS_CURRENT: LazyLock<Mutex<Vec<qbz_library::LocalTrack>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

pub(crate) fn tracks_current() -> std::sync::MutexGuard<'static, Vec<qbz_library::LocalTrack>> {
    TRACKS_CURRENT.lock().unwrap_or_else(|e| e.into_inner())
}

/// Snapshot of the currently-loaded Tracks-tab rows (already carry their
/// covers). Used to build the play queue instantly on a row click — avoiding
/// the full DB re-query + cover-fill that delayed queue population.
pub fn tracks_current_snapshot() -> Vec<qbz_library::LocalTrack> {
    tracks_current().clone()
}

/// Fetch one tracks page off the UI thread. `LocalTrack` is Send, so it
/// crosses the `spawn_blocking` boundary; the conversion to `TrackItem`
/// happens on the UI thread. has_more = the LOCAL page came back full.
pub(crate) fn fetch_tracks_page(
    query: String,
    offset: u64,
    sort: String,
) -> Option<(Vec<qbz_library::LocalTrack>, bool)> {
    // exclude_network_folders: connectivity-keyed — see the NETWORK-FOLDER
    // VISIBILITY note.
    let exclude_network = exclude_network_folders_now();
    let rows = crate::library_db::with_db(|db| {
        db.search_with_filter_page(query.trim(), offset, TRACKS_PAGE, true, exclude_network, &sort)
    })?;
    let has_more = rows.len() as u64 == TRACKS_PAGE;
    Some((rows, has_more))
}
