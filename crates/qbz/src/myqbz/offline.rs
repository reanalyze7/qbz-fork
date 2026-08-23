//! Offline availability (D11.c): the cached Qobuz track + album id sets and
//! the D4 grace verdict, used to filter mixtape/collection items + rows.

use std::collections::HashSet;

use qbz_models::mixtape::{AlbumSource, ItemType, MixtapeCollection, MixtapeCollectionItem};

/// Offline availability snapshot for Mixtape/Collection items: the cached
/// Qobuz track + album id sets (ONE batch read of the offline index) and the
/// D4 grace verdict. Built per grid/detail load WHILE OFFLINE only — online
/// nothing is filtered and this is never constructed.
pub struct OfflineAvailability {
    cached_track_ids: HashSet<u64>,
    cached_album_ids: HashSet<String>,
    /// Qobuz cache may serve full tracks (D4 grace window).
    qobuz_allowed: bool,
}

impl OfflineAvailability {
    /// The availability rule, per item:
    /// local → available; qobuz → offline-cached AND within grace. Qobuz
    /// playlists (membership lives in the API) and the unsupported
    /// local-playlist items resolve to nothing offline → hidden.
    pub fn item_available(&self, item: &MixtapeCollectionItem) -> bool {
        match item.source {
            AlbumSource::Qobuz => {
                if !self.qobuz_allowed {
                    return false;
                }
                match item.item_type {
                    ItemType::Album => self.cached_album_ids.contains(&item.source_item_id),
                    ItemType::Track => item
                        .source_item_id
                        .parse::<u64>()
                        .map(|id| self.cached_track_ids.contains(&id))
                        .unwrap_or(false),
                    // Membership is API-side — not enumerable offline.
                    ItemType::Playlist => false,
                }
            }
            AlbumSource::Local => match item.item_type {
                ItemType::Track => item.source_item_id.parse::<i64>().is_ok(),
                // The resolver rejects local playlists outright.
                ItemType::Playlist => false,
                ItemType::Album => true,
            },
        }
    }
}

/// Build the snapshot for `items`. One async batch read of the offline
/// index (cached track + album ids); the grace flag is a cheap status read.
pub async fn offline_availability(_items: &[&MixtapeCollectionItem]) -> OfflineAvailability {
    let (cached_track_ids, cached_album_ids) = match crate::offline::get().await {
        Some(off) => {
            let guard = off.db.lock().await;
            match guard.as_ref().map(|db| db.get_all_tracks()) {
                Some(Ok(tracks)) => {
                    let mut ids = HashSet::new();
                    let mut albums = HashSet::new();
                    for t in tracks {
                        if matches!(t.status, qbz_offline_cache::OfflineCacheStatus::Ready) {
                            ids.insert(t.track_id);
                            if let Some(album_id) = t.album_id {
                                albums.insert(album_id);
                            }
                        }
                    }
                    (ids, albums)
                }
                _ => (HashSet::new(), HashSet::new()),
            }
        }
        None => (HashSet::new(), HashSet::new()),
    };

    OfflineAvailability {
        cached_track_ids,
        cached_album_ids,
        qobuz_allowed: crate::offline_mode::offline_playback_allowed(),
    }
}

/// D11.c grid filter: drop each collection's unavailable items, then drop
/// collections left with ZERO items. Counts + mosaics + the detail stay
/// consistent (they all derive from the filtered item set). Offline only.
pub async fn retain_available_offline(rows: Vec<MixtapeCollection>) -> Vec<MixtapeCollection> {
    let items: Vec<&MixtapeCollectionItem> =
        rows.iter().flat_map(|c| c.items.iter()).collect();
    let avail = offline_availability(&items).await;
    drop(items);
    rows.into_iter()
        .filter_map(|mut c| {
            c.items.retain(|it| avail.item_available(it));
            (!c.items.is_empty()).then_some(c)
        })
        .collect()
}
