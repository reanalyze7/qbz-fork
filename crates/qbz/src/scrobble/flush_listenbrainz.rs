//! Flush the ListenBrainz offline listen queue from the shared cache.

use qbz_integrations::listenbrainz::cache::ListenBrainzCache;
use qbz_integrations::listenbrainz::AdditionalInfo;
use qbz_integrations::ListenBrainzClient;

use crate::scrobbler_settings;

use super::listenbrainz_cache_path;

/// Flush the ListenBrainz queue from the shared cache. Stops at the first
/// failure and retries on the next edge.
pub(super) async fn flush_listenbrainz_queue() {
    let cfg = scrobbler_settings::get();
    if !cfg.listenbrainz_is_authed() {
        return;
    }
    let Some(path) = listenbrainz_cache_path() else {
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

    let client = ListenBrainzClient::new();
    client
        .restore_token(cfg.listenbrainz_token.clone(), cfg.listenbrainz_username.clone())
        .await;
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
            break; // still failing — retry on the next edge
        }
    }
    if !sent_ids.is_empty() {
        let count = sent_ids.len();
        let _ = tokio::task::spawn_blocking(move || {
            ListenBrainzCache::new(&path).and_then(|c| c.mark_listens_sent(&sent_ids))
        })
        .await;
        log::info!("[qbz-slint] ListenBrainz flush: {count} listen(s) sent");
    }
}
