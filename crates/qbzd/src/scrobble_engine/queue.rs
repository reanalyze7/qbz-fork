use std::path::PathBuf;

use qbz_app::settings::scrobblers::ScrobblerSettings;
use qbz_integrations::listenbrainz::cache::ListenBrainzCache;
use qbz_integrations::listenbrainz::AdditionalInfo;
use qbz_models::QueueTrack;

use crate::paths::ProfileRoots;

use super::providers::lb_client;
use super::pure::album_opt;

/// Persist a failed listen into the SHARED `ListenBrainzCache.listen_queue`
/// (daemon-root `cache/listenbrainz_v2.db`). Opened inside a `spawn_blocking` so
/// the rusqlite Connection never crosses an await.
pub(super) async fn queue_listenbrainz(roots: &ProfileRoots, t: &QueueTrack, timestamp: i64) {
    let Some(path) = lb_cache_path(roots) else {
        return;
    };
    let artist = t.artist.clone();
    let track = t.title.clone();
    let album = album_opt(t).map(str::to_string);
    let duration_ms = (t.duration_secs > 0).then_some(t.duration_secs * 1000);
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
                log::warn!("[scrobbler] queue listenbrainz listen failed: {e}");
            }
        }
        Err(e) => log::warn!("[scrobbler] open listenbrainz cache failed: {e}"),
    })
    .await;
}

/// Drain pending ListenBrainz listens oldest-first, stopping at the first
/// failure (still offline / flaky — retry on the next tick). Mirrors
/// `qbz::scrobble::flush_listenbrainz_queue`.
pub(super) async fn drain_listenbrainz(s: &ScrobblerSettings, roots: &ProfileRoots) {
    let Some(path) = lb_cache_path(roots) else {
        return;
    };
    let pending = match tokio::task::spawn_blocking({
        let path = path.clone();
        move || ListenBrainzCache::new(&path).and_then(|c| c.get_pending_listens(500))
    })
    .await
    {
        Ok(Ok(p)) => p,
        _ => return,
    };
    if pending.is_empty() {
        return;
    }

    let client = lb_client(s);
    let mut sent_ids: Vec<i64> = Vec::new();
    for item in pending {
        let info = AdditionalInfo {
            recording_mbid: item.recording_mbid.clone(),
            release_mbid: item.release_mbid.clone(),
            artist_mbids: item.artist_mbids.clone(),
            isrc: item.isrc.clone(),
            duration_ms: item.duration_ms,
            ..Default::default()
        };
        if client
            .submit_listen(
                &item.artist_name,
                &item.track_name,
                item.release_name.as_deref(),
                item.listened_at,
                Some(info),
            )
            .await
            .is_ok()
        {
            sent_ids.push(item.id);
        } else {
            break; // still failing — retry on the next tick
        }
    }
    if !sent_ids.is_empty() {
        let count = sent_ids.len();
        let _ = tokio::task::spawn_blocking(move || {
            ListenBrainzCache::new(&path).and_then(|c| c.mark_listens_sent(&sent_ids))
        })
        .await;
        log::info!("[scrobbler] listenbrainz drain: {count} listen(s) sent");
    }
}

/// The daemon-root shared ListenBrainz cache DB — `<cache>/listenbrainz_v2.db`,
/// the same file name and schema the desktop opens (the daemon uses its own
/// `qbzd` cache root; it never touches the desktop's dirs).
fn lb_cache_path(roots: &ProfileRoots) -> Option<PathBuf> {
    std::fs::create_dir_all(&roots.cache).ok()?;
    Some(roots.cache.join("listenbrainz_v2.db"))
}
