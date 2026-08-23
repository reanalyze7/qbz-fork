// crates/qbzd/src/scrobble_engine/ — scrobble-on-play (CONSOLE ext).
//
// A daemon background task subscribing the DaemonAdapter CoreEvent bus. On
// `TrackStarted` it sends "now playing" to every ACTIVE provider; when the
// track crosses the scrobble threshold (`qbz_app::scrobble_timing::
// scrobble_delay_secs` — Last.fm's played-half-or-4-min rule) it scrobbles
// ONCE. Credentials are re-read from the canonical `ScrobblerSettingsStore` on
// each track start, so `qbzd scrobble …` changes take effect on the next track
// with no reload signal. Best-effort + logged.
//
// Providers: Last.fm (LastFmClient::update_now_playing / scrobble) and
// ListenBrainz (submit_playing_now / submit_listen). Both backends are
// qbz-integrations (Slint-free).
//
// ListenBrainz has a persistent offline queue: a failed `submit_listen` is
// written to the SHARED `ListenBrainzCache.listen_queue` (daemon-root
// `cache/listenbrainz_v2.db`, the same schema the desktop uses) and a periodic
// drain — plus one drain at task start — retries pending listens oldest-first,
// stopping at the first failure and resuming on the next tick. The rusqlite
// Connection is never held across an await: it is opened inside a
// `spawn_blocking` for each queue/drain op (mirrors `qbz::scrobble`).
mod providers;
mod pure;
mod queue;
#[cfg(test)]
mod tests;

use std::time::Duration;

use qbz_app::settings::scrobblers::ScrobblerSettingsStore;
use qbz_models::{CoreEvent, QueueTrack};
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use crate::paths::ProfileRoots;

use providers::{now_playing, scrobble};
use pure::{due, now_unix};
use queue::drain_listenbrainz;

/// How often the ListenBrainz offline queue is retried (plus once at task
/// start). Live now-playing/scrobble submits are unaffected — the drain only
/// clears listens that a prior submit could not deliver.
const DRAIN_INTERVAL: Duration = Duration::from_secs(120);

/// The track currently being timed for a scrobble.
struct Playing {
    track: QueueTrack,
    /// Unix seconds when it started — Last.fm's scrobble timestamp.
    started_at: u64,
    /// Seconds into the track at which it becomes scrobble-eligible; `None`
    /// means "too short to scrobble" (`scrobble_delay_secs` returned None).
    threshold: Option<u64>,
    scrobbled: bool,
}

/// Spawn the scrobble-on-play task. Holds NO `Arc<AppRuntime>` (only the roots,
/// its own store, and the bus receiver), so it is outside the §8.2 audio
/// clock-release ordering — the caller aborts it for a clean shutdown.
pub fn spawn(roots: ProfileRoots, mut rx: broadcast::Receiver<CoreEvent>) -> JoinHandle<()> {
    use broadcast::error::RecvError;
    tokio::spawn(async move {
        let store = match ScrobblerSettingsStore::new_at(&roots.data) {
            Ok(s) => s,
            Err(e) => {
                log::warn!("[scrobbler] store open failed; scrobbling disabled: {e}");
                return;
            }
        };
        let mut playing: Option<Playing> = None;
        // Fires immediately on the first tick (drains any queue left from a
        // prior offline session), then every DRAIN_INTERVAL.
        let mut drain = tokio::time::interval(DRAIN_INTERVAL);
        drain.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            // `biased`: live bus events take priority over the drain tick.
            tokio::select! {
                biased;
                ev = rx.recv() => match ev {
                    Ok(CoreEvent::TrackStarted { track, .. }) => {
                        let settings = store.get_settings().unwrap_or_default();
                        if !settings.enabled {
                            playing = None;
                            continue;
                        }
                        now_playing(&settings, &track).await;
                        playing = Some(Playing {
                            threshold: qbz_app::scrobble_timing::scrobble_delay_secs(track.duration_secs),
                            started_at: now_unix(),
                            track,
                            scrobbled: false,
                        });
                    }
                    Ok(CoreEvent::PositionUpdated { position_secs, .. }) => {
                        if let Some(p) = playing.as_mut() {
                            if due(position_secs, p.threshold, p.scrobbled) {
                                let settings = store.get_settings().unwrap_or_default();
                                scrobble(&settings, &p.track, p.started_at, &roots).await;
                                p.scrobbled = true;
                            }
                        }
                    }
                    Ok(_) => {}
                    Err(RecvError::Lagged(_)) => continue,
                    Err(RecvError::Closed) => return,
                },
                _ = drain.tick() => {
                    let settings = store.get_settings().unwrap_or_default();
                    if settings.enabled && settings.listenbrainz_active() {
                        drain_listenbrainz(&settings, &roots).await;
                    }
                }
            }
        }
    })
}
