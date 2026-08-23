//! Fire "now playing" and the actual scrobble to each enabled service.

use qbz_integrations::{LastFmClient, ListenBrainzClient};

use crate::scrobbler_settings;

use super::fire::{lb_info, ScrobbleMeta};
use super::queue::{queue_lastfm, queue_listenbrainz};

/// Fire "now playing" for each enabled service. Failures only log — the
/// scrobble path is what queues.
pub(super) async fn send_now_playing(meta: &ScrobbleMeta, cfg: &scrobbler_settings::ScrobblerSettings) {
    let album = meta.album.as_deref();
    if cfg.lastfm_active() {
        let client = LastFmClient::with_session_key(cfg.lastfm_session_key.clone());
        if let Err(e) = client
            .update_now_playing(&meta.artist, &meta.track, album)
            .await
        {
            log::debug!("[qbz-slint] Last.fm now-playing failed: {e}");
        }
    }
    if cfg.listenbrainz_active() {
        let client = ListenBrainzClient::new();
        client
            .restore_token(cfg.listenbrainz_token.clone(), cfg.listenbrainz_username.clone())
            .await;
        if let Err(e) = client
            .submit_playing_now(&meta.artist, &meta.track, album, lb_info(meta.duration_secs))
            .await
        {
            log::debug!("[qbz-slint] ListenBrainz now-playing failed: {e}");
        }
    }
}

/// Fire the actual scrobble for each enabled service. Engine offline OR call
/// failure queues it — Last.fm to the shared `scrobble_queue`, ListenBrainz to
/// the shared `listen_queue`. Re-reads settings in case the user disconnected
/// while the timer waited.
pub(super) async fn send_scrobble(meta: &ScrobbleMeta) {
    let cfg = scrobbler_settings::get();
    let album = meta.album.as_deref();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let offline = crate::offline_mode::engine().is_offline();

    if cfg.lastfm_active() {
        let sent = if offline {
            false
        } else {
            let client = LastFmClient::with_session_key(cfg.lastfm_session_key.clone());
            match client
                .scrobble(&meta.artist, &meta.track, album, timestamp as u64)
                .await
            {
                Ok(()) => {
                    log::info!(
                        "[qbz-slint] Last.fm scrobbled: {} - {}",
                        meta.artist,
                        meta.track
                    );
                    true
                }
                Err(e) => {
                    log::warn!("[qbz-slint] Last.fm scrobble failed ({e}); queueing for later");
                    false
                }
            }
        };
        if !sent {
            queue_lastfm(meta, timestamp).await;
        }
    }

    if cfg.listenbrainz_active() {
        let sent = if offline {
            false
        } else {
            let client = ListenBrainzClient::new();
            client
                .restore_token(cfg.listenbrainz_token.clone(), cfg.listenbrainz_username.clone())
                .await;
            match client
                .submit_listen(
                    &meta.artist,
                    &meta.track,
                    album,
                    timestamp,
                    lb_info(meta.duration_secs),
                )
                .await
            {
                Ok(()) => {
                    log::info!(
                        "[qbz-slint] ListenBrainz scrobbled: {} - {}",
                        meta.artist,
                        meta.track
                    );
                    true
                }
                Err(e) => {
                    log::warn!(
                        "[qbz-slint] ListenBrainz scrobble failed ({e}); queueing for later"
                    );
                    false
                }
            }
        };
        if !sent {
            queue_listenbrainz(meta, timestamp).await;
        }
    }
}
