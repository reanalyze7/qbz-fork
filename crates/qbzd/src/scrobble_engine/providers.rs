use qbz_app::settings::scrobblers::ScrobblerSettings;
use qbz_integrations::lastfm::LastFmClient;
use qbz_integrations::listenbrainz::{ListenBrainzClient, ListenBrainzConfig};
use qbz_models::QueueTrack;

use crate::paths::ProfileRoots;

use super::pure::album_opt;
use super::queue::queue_listenbrainz;

pub(super) async fn now_playing(s: &ScrobblerSettings, t: &QueueTrack) {
    let album = album_opt(t);
    if s.lastfm_active() {
        let c = LastFmClient::with_session_key(s.lastfm_session_key.clone());
        if let Err(e) = c.update_now_playing(&t.artist, &t.title, album).await {
            log::debug!("[scrobbler] last.fm now-playing failed: {e}");
        }
    }
    if s.listenbrainz_active() {
        let c = lb_client(s);
        if let Err(e) = c.submit_playing_now(&t.artist, &t.title, album, None).await {
            log::debug!("[scrobbler] listenbrainz now-playing failed: {e}");
        }
    }
}

pub(super) async fn scrobble(
    s: &ScrobblerSettings,
    t: &QueueTrack,
    started_at: u64,
    roots: &ProfileRoots,
) {
    let album = album_opt(t);
    if s.lastfm_active() {
        let c = LastFmClient::with_session_key(s.lastfm_session_key.clone());
        match c.scrobble(&t.artist, &t.title, album, started_at).await {
            Ok(()) => log::info!("[scrobbler] last.fm scrobbled: {} — {}", t.artist, t.title),
            Err(e) => log::warn!("[scrobbler] last.fm scrobble failed: {e}"),
        }
    }
    if s.listenbrainz_active() {
        let c = lb_client(s);
        match c.submit_listen(&t.artist, &t.title, album, started_at as i64, None).await {
            Ok(()) => log::info!("[scrobbler] listenbrainz submitted: {} — {}", t.artist, t.title),
            Err(e) => {
                // Persist to the shared offline queue; the periodic drain retries it.
                log::warn!("[scrobbler] listenbrainz submit failed, queueing: {e}");
                queue_listenbrainz(roots, t, started_at as i64).await;
            }
        }
    }
}

/// A ListenBrainz client bound to the stored token, with its own enabled flag
/// ON (submit_* early-returns if the client config is disabled — our gate is
/// `ScrobblerSettings::listenbrainz_active`, checked before calling).
pub(super) fn lb_client(s: &ScrobblerSettings) -> ListenBrainzClient {
    ListenBrainzClient::with_config(ListenBrainzConfig {
        enabled: true,
        token: Some(s.listenbrainz_token.clone()),
        user_name: Some(s.listenbrainz_username.clone()),
    })
}
