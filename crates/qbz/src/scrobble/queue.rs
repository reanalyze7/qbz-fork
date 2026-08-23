//! Write a failed/offline fire into the shared per-user offline queues.

use qbz_app::offline_mode::OfflineModeStore;
use qbz_integrations::listenbrainz::cache::ListenBrainzCache;

use super::fire::ScrobbleMeta;
use super::listenbrainz_cache_path;
use crate::scrobbler_settings;

/// Queue a Last.fm scrobble into the SHARED per-user `offline_settings.db`
/// `scrobble_queue` (the table Tauri queues into and flushes from).
pub(super) async fn queue_lastfm(meta: &ScrobbleMeta, timestamp: i64) {
    let Some(dir) = scrobbler_settings::user_dir() else {
        return;
    };
    let artist = meta.artist.clone();
    let track = meta.track.clone();
    let album = meta.album.clone();
    let _ = tokio::task::spawn_blocking(move || {
        match OfflineModeStore::new_at(&dir) {
            Ok(store) => {
                if let Err(e) = store.queue_scrobble(&artist, &track, album.as_deref(), timestamp)
                {
                    log::warn!("[qbz-slint] queue Last.fm scrobble failed: {e}");
                }
            }
            Err(e) => log::warn!("[qbz-slint] open offline settings store failed: {e}"),
        }
    })
    .await;
}

/// Queue a ListenBrainz listen into the SHARED per-user
/// `ListenBrainzCache.listen_queue` (the canonical LB offline store).
pub(super) async fn queue_listenbrainz(meta: &ScrobbleMeta, timestamp: i64) {
    let Some(path) = listenbrainz_cache_path() else {
        return;
    };
    let artist = meta.artist.clone();
    let track = meta.track.clone();
    let album = meta.album.clone();
    let duration_ms = (meta.duration_secs > 0).then_some(meta.duration_secs * 1000);
    let _ = tokio::task::spawn_blocking(move || match ListenBrainzCache::new(&path) {
        Ok(cache) => {
            if let Err(e) = cache.queue_listen(
                timestamp,
                &artist,
                &track,
                album.as_deref(),
                None,
                None,
                None,
                None,
                duration_ms,
            ) {
                log::warn!("[qbz-slint] queue ListenBrainz listen failed: {e}");
            }
        }
        Err(e) => log::warn!("[qbz-slint] open ListenBrainz cache failed: {e}"),
    })
    .await;
}
