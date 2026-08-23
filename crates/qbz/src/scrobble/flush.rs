//! Draining both offline queues on reconnect / shell entry.
//!
//! Offline flush — drain both queues (shell entry + every offline->online
//! edge).

use qbz_app::offline_mode::OfflineModeStore;
use qbz_integrations::LastFmClient;

use crate::scrobbler_settings;

use super::flush_listenbrainz::flush_listenbrainz_queue;

pub(super) async fn flush_offline_queues() {
    flush_lastfm_queue().await;
    flush_listenbrainz_queue().await;
}

/// Flush the Last.fm queue: up to 50 per pass (the Last.fm batch limit),
/// oldest first; entries older than 14 days are dropped (marked sent) since
/// Last.fm rejects them — both mirror the Svelte `flushScrobbleQueue`. Stops
/// at the first network failure (still offline / flaky) and retries on the
/// next edge. Cleans up sent rows older than 7 days afterwards.
async fn flush_lastfm_queue() {
    let cfg = scrobbler_settings::get();
    if !cfg.lastfm_is_authed() {
        return;
    }
    let Some(dir) = scrobbler_settings::user_dir() else {
        return;
    };
    let pending = match tokio::task::spawn_blocking({
        let dir = dir.clone();
        move || OfflineModeStore::new_at(&dir).and_then(|s| s.get_queued_scrobbles(50))
    })
    .await
    {
        Ok(Ok(p)) => p,
        _ => return,
    };
    if pending.is_empty() {
        return;
    }

    let client = LastFmClient::with_session_key(cfg.lastfm_session_key.clone());
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let cutoff = now - 14 * 86400;
    let mut sent_ids: Vec<i64> = Vec::new();
    for item in pending {
        if item.timestamp < cutoff {
            // Too old for Last.fm — drop it (mark sent so it stops re-trying).
            sent_ids.push(item.id);
            continue;
        }
        match client
            .scrobble(
                &item.artist,
                &item.track,
                item.album.as_deref(),
                item.timestamp as u64,
            )
            .await
        {
            Ok(()) => sent_ids.push(item.id),
            Err(e) => {
                log::warn!(
                    "[qbz-slint] Last.fm flush stopped at {} - {}: {e}",
                    item.artist,
                    item.track
                );
                break; // still offline / failing — retry on the next edge
            }
        }
    }
    if !sent_ids.is_empty() {
        let count = sent_ids.len();
        let _ = tokio::task::spawn_blocking(move || {
            OfflineModeStore::new_at(&dir).and_then(|s| {
                s.mark_scrobbles_sent(&sent_ids)?;
                s.cleanup_sent_scrobbles(7)
            })
        })
        .await;
        log::info!("[qbz-slint] Last.fm flush: {count} scrobble(s) sent/cleared");
    }
}
