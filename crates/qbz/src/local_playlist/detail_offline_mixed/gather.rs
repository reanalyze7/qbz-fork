use crate::local_playlist::detail_local::read_sidecar_rows_blocking;
use crate::local_playlist::row::{LoadedRow, RowItem};

/// Blocking gather: the sidecar rows (healed + position-sorted), the custom
/// artwork path, the B8 playable snapshot ids, and the persisted snapshot
/// name. Run inside `spawn_blocking`.
pub(super) fn gather_sidecar(
    playlist_id: u64,
) -> (Vec<LoadedRow>, Option<String>, Vec<u64>, Option<String>) {
    // Healing base: the best offline guess at the Qobuz block size (sidebar
    // session cache, else the B7 snapshot count).
    let qobuz_count = crate::sidebar::playlist_track_count(playlist_id)
        .or_else(|| {
            crate::playlist_snapshot::headers_blocking()
                .get(&playlist_id)
                .and_then(|(_, count)| *count)
        })
        .unwrap_or(0);
    // Shared sidecar reader (heals positions, honest Unresolved fallbacks).
    let mut rows = read_sidecar_rows_blocking(playlist_id, qobuz_count);
    // Sidecar block in one position order (the merge's claim order).
    rows.sort_by_key(|r| r.position);
    let custom = crate::library_db::with_db(|db| {
        Ok(db
            .get_playlist_settings(playlist_id)?
            .and_then(|s| s.custom_artwork_path)
            .filter(|p| !p.is_empty()))
    })
    .flatten();
    (
        rows,
        custom,
        crate::playlist_snapshot::playable_track_ids_blocking(playlist_id),
        crate::playlist_snapshot::name_blocking(playlist_id),
    )
}

/// B8: resolve the playable snapshot ids against the offline-cache index
/// (metadata + B5 cover chain), keeping snapshot order. Ids whose copy
/// vanished since the cached-set check resolve to nothing and drop,
/// mirroring the LOCAL detail's D11 filter.
pub(super) async fn resolve_playable(playable_ids: &[u64]) -> Vec<LoadedRow> {
    let mut rows: Vec<LoadedRow> = Vec::new();
    if playable_ids.is_empty() {
        return rows;
    }
    let Some(off) = crate::offline::get().await else {
        return rows;
    };
    let cache_path = off.get_cache_path();
    let guard = off.db.lock().await;
    let Some(db) = guard.as_ref() else {
        return rows;
    };
    for (i, tid) in playable_ids.iter().enumerate() {
        if let Ok(Some(info)) = db.get_track(*tid) {
            if matches!(info.status, qbz_offline_cache::OfflineCacheStatus::Ready) {
                let artwork_path = info.resolve_cover_path(&cache_path);
                rows.push(LoadedRow {
                    position: i as i32,
                    item: RowItem::Cached {
                        track_id: info.track_id,
                        title: info.title,
                        artist: info.artist,
                        album: info.album.unwrap_or_default(),
                        duration_secs: info.duration_secs,
                        bit_depth: info.bit_depth,
                        sample_rate: info.sample_rate,
                        artwork_path,
                    },
                });
            }
        }
    }
    rows
}
