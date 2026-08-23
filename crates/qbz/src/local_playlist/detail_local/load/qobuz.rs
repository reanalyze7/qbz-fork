use std::collections::HashMap;

use crate::local_playlist::row::RowItem;
use crate::local_playlist::Runtime;

/// Qobuz rows: one batch fetch when online; cached-metadata fallback for
/// everything the batch did not return (and for the whole set offline).
pub(super) async fn resolve_qobuz(
    runtime: &Runtime,
    id: &str,
    qobuz_ids: &[u64],
    offline: bool,
) -> (HashMap<u64, qbz_models::Track>, HashMap<u64, RowItem>) {
    let mut fetched: HashMap<u64, qbz_models::Track> = HashMap::new();
    if !offline && !qobuz_ids.is_empty() {
        match runtime.core().get_tracks_batch(qobuz_ids).await {
            Ok(list) => {
                for t in list {
                    fetched.insert(t.id, t);
                }
            }
            Err(e) => {
                log::warn!("[qbz-slint] local playlist {id}: qobuz batch failed: {e}");
            }
        }
    }
    let missing: Vec<u64> = qobuz_ids
        .iter()
        .copied()
        .filter(|tid| !fetched.contains_key(tid))
        .collect();
    let mut cached: HashMap<u64, RowItem> = HashMap::new();
    if !missing.is_empty() {
        if let Some(off) = crate::offline::get().await {
            let cache_path = off.get_cache_path();
            let guard = off.db.lock().await;
            if let Some(db) = guard.as_ref() {
                for tid in &missing {
                    if let Ok(Some(info)) = db.get_track(*tid) {
                        if matches!(info.status, qbz_offline_cache::OfflineCacheStatus::Ready) {
                            let artwork_path = info.resolve_cover_path(&cache_path);
                            cached.insert(
                                *tid,
                                RowItem::Cached {
                                    track_id: info.track_id,
                                    title: info.title,
                                    artist: info.artist,
                                    album: info.album.unwrap_or_default(),
                                    duration_secs: info.duration_secs,
                                    bit_depth: info.bit_depth,
                                    sample_rate: info.sample_rate,
                                    artwork_path,
                                },
                            );
                        }
                    }
                }
            }
        }
    }
    (fetched, cached)
}
