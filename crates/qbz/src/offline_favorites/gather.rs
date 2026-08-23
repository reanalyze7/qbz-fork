//! Async gather of the rail's rows + queue from the three local stores.

use std::collections::HashSet;

use qbz_models::QueueTrack;
use qbz_offline_cache::{CachedTrackInfo, OfflineCacheStatus};

use super::state::{index_queue_track, RowData, COVER_DECODE_SIZE};

/// Read favorites + the offline-cache index + the library "Offline"
/// source-filter rows, and build the rail's display rows + play queue in
/// display order (index rows first — recency — then library-only copies).
pub(super) async fn gather() -> (Vec<RowData>, Vec<QueueTrack>) {
    let favorites: HashSet<u64> = crate::fav_cache::all();

    // Offline-cache index: READY rows (most-recently-accessed first,
    // the DB's order) + the cache root for the cover chain.
    let (index_rows, cache_path): (Vec<CachedTrackInfo>, String) = match crate::offline::get().await
    {
        Some(off) => {
            let cp = off.get_cache_path();
            let guard = off.db.lock().await;
            let rows: Vec<CachedTrackInfo> = guard
                .as_ref()
                .and_then(|db| db.get_all_tracks().ok())
                .unwrap_or_default()
                .into_iter()
                .filter(|t| matches!(t.status, OfflineCacheStatus::Ready))
                .collect();
            (rows, cp)
        }
        None => (Vec::new(), String::new()),
    };

    // library.db qobuz_download rows (the LocalLibrary "Offline"
    // source-filter set). `with_db` opens the file on the current
    // thread, so it runs inside spawn_blocking.
    let mut lib_rows: Vec<qbz_library::LocalTrack> = tokio::task::spawn_blocking(|| {
        crate::library_db::with_db(|db| db.get_qobuz_download_tracks()).unwrap_or_default()
    })
    .await
    .unwrap_or_default();
    // Folder-cover backfill: the downloaders write cover.jpg next to
    // the file without always backfilling artwork_path.
    crate::playback::fill_missing_covers(&mut lib_rows);

    // Intersection, in display order: index rows first (recency), then
    // library-only copies. The index row wins metadata when an id is in
    // both (richer quality columns + the offline cover chain).
    let mut seen: HashSet<u64> = HashSet::new();
    let mut rows: Vec<RowData> = Vec::new();
    let mut queue: Vec<QueueTrack> = Vec::new();
    for t in &index_rows {
        if !favorites.contains(&t.track_id) || !seen.insert(t.track_id) {
            continue;
        }
        let cover = t.resolve_cover_path(&cache_path).unwrap_or_default();
        queue.push(index_queue_track(t, &cover));
        rows.push(RowData {
            id: t.track_id.to_string(),
            title: t.title.clone(),
            artist: t.artist.clone(),
            cover: crate::artwork::decode_local_pixels(
                &cover,
                crate::artwork::scaled_decode(COVER_DECODE_SIZE),
            ),
        });
    }
    for lt in &lib_rows {
        let Some(qid) = lt.qobuz_track_id.and_then(|v| u64::try_from(v).ok()) else {
            continue;
        };
        if !favorites.contains(&qid) || !seen.insert(qid) {
            continue;
        }
        queue.push(crate::playback::local_queue_track(lt));
        rows.push(RowData {
            id: qid.to_string(),
            title: lt.title.clone(),
            artist: lt.artist.clone(),
            cover: crate::artwork::decode_local_pixels(
                lt.artwork_path.as_deref().unwrap_or_default(),
                crate::artwork::scaled_decode(COVER_DECODE_SIZE),
            ),
        });
    }

    // Playable favorites with no local metadata row are skipped by
    // construction (membership comes FROM the metadata rows); keep the
    // count observable per the backlog contract.
    let ready_ids: HashSet<u64> = index_rows.iter().map(|t| t.track_id).collect();
    let lib_ids: HashSet<u64> = lib_rows
        .iter()
        .filter_map(|t| t.qobuz_track_id.and_then(|v| u64::try_from(v).ok()))
        .collect();
    let playable = favorites
        .iter()
        .filter(|id| ready_ids.contains(id) || lib_ids.contains(id))
        .count();
    let skipped = playable.saturating_sub(rows.len());
    log::info!(
        "[qbz-slint] offline favorites rail: {} playable of {} favorites ({} skipped — no local metadata)",
        rows.len(),
        favorites.len(),
        skipped
    );

    (rows, queue)
}
