//! The now-playing + delayed-scrobble firing entry point.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use qbz_integrations::listenbrainz::AdditionalInfo;

use crate::scrobbler_settings;

use super::fire_send::send_now_playing;
use super::rt_handle;

/// Pure arming rule lives in `qbz-app` so unit tests do not need the Slint
/// binary compile (see `qbz_app::scrobble_timing`).
use qbz_app::scrobble_timing::scrobble_delay_secs;

/// Normalized track facts the fire path needs. Built from the CURRENT
/// `QueueTrack` on the de-duped track-change edge; the title is the
/// version-enriched display title so remixes/editions scrobble correctly
/// (issue #360 parity with the Svelte `formatTrackTitle` path).
#[derive(Clone)]
pub struct ScrobbleMeta {
    pub artist: String,
    pub track: String,
    /// `None` when empty — clients take `Option<&str>` for album.
    pub album: Option<String>,
    pub duration_secs: u64,
}

/// Monotonic generation, bumped on every track change so a delayed scrobble
/// timer that fires after the user skipped is dropped (the Svelte
/// `clearTimeout` equivalent). Like Tauri, pause/stop do NOT cancel it.
pub(super) static SCROBBLE_GEN: AtomicU64 = AtomicU64::new(0);

/// Track-change entry point. Fires now-playing immediately for each enabled +
/// authed service, then arms a delayed scrobble. No-op when no service is
/// active. Called from `refresh_now_playing_meta` on the de-duped track-change
/// edge (after the QConnect peer-active gate), so it is NOT re-armed on
/// resume/seek and never fires for a remote renderer's audio.
pub fn on_track_changed(meta: ScrobbleMeta) {
    // Always bump the generation so any in-flight stale timer self-cancels.
    let my_gen = SCROBBLE_GEN.fetch_add(1, Ordering::SeqCst) + 1;
    let cfg = scrobbler_settings::get();
    if !cfg.lastfm_active() && !cfg.listenbrainz_active() {
        return;
    }
    let Some(handle) = rt_handle() else {
        return;
    };
    handle.spawn(async move {
        // Now-playing immediately (skipped while offline — needs network and
        // is not worth queueing; matches the Svelte path).
        if !crate::offline_mode::engine().is_offline() {
            send_now_playing(&meta, &cfg).await;
        }

        // Delayed scrobble at min(dur/2, 240s). Unknown duration: skip scrobble
        // (still sent now-playing above when online).
        let Some(wait) = scrobble_delay_secs(meta.duration_secs) else {
            log::debug!(
                "[scrobble] skip delayed scrobble: unknown duration for '{}'",
                meta.track
            );
            return;
        };
        if wait > 0 {
            tokio::time::sleep(Duration::from_secs(wait)).await;
        }
        // Self-cancel if a newer track change superseded us.
        if SCROBBLE_GEN.load(Ordering::SeqCst) != my_gen {
            return;
        }
        super::fire_send::send_scrobble(&meta).await;
    });
}

/// Optional ListenBrainz extras — duration is the only one the QueueTrack
/// carries (no ISRC / MB IDs on the queue model yet).
pub(super) fn lb_info(duration_secs: u64) -> Option<AdditionalInfo> {
    Some(AdditionalInfo {
        duration_ms: (duration_secs > 0).then_some(duration_secs * 1000),
        ..Default::default()
    })
}
