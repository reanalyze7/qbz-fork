//! Audible-playback engine: the core "fetch bytes -> hand to Player ->
//! update UI" pipeline shared by every play path.

use qbz_models::Quality;

use super::meta::{record_recent, refresh_now_playing_meta};
use super::quality::local_playback_quality;
use super::Runtime;
use crate::AppWindow;

mod offline_gate;
mod play_audible;
pub(super) use play_audible::play_audible;

/// Shared post-track-change step: update the now-playing card, record the
/// play in the recently-played store, and start audio for `track_id`.
/// Used by the queue controller's play paths.
pub async fn after_track_change(runtime: &Runtime, weak: &slint::Weak<AppWindow>, track_id: u64) {
    refresh_now_playing_meta(runtime, weak).await;
    record_recent(runtime).await;
    play_audible(runtime, weak, track_id).await;
    // Warm the cache for the upcoming tracks so the next transition can be
    // gapless (a cached track plays via `play_data`, which the audio
    // engine's gapless engine supports; a streamed track does not).
    kick_prefetch(runtime).await;
    // Persist the session (queue + current track + position) so a restart can
    // restore it. No-op unless `persist_session` is on.
    crate::session_persist::capture_and_save(runtime).await;
}

/// How many upcoming queue tracks to prefetch into the player cache.
/// Two tracks ahead is enough headroom for gapless without holding an
/// excessive number of HiRes payloads in memory. Matches the spirit of
/// Tauri's `v2_prefetch_count` (which is host-tuned; the Slint MVP uses
/// a fixed small value).
const PREFETCH_LOOKAHEAD: usize = 2;

/// Maximum concurrent prefetch downloads — mirrors Tauri's
/// `v2_max_concurrent_prefetch` default for normal hosts.
const MAX_CONCURRENT_PREFETCH: usize = 2;

/// Shared semaphore bounding concurrent prefetch downloads across all
/// `kick_prefetch` calls.
static PREFETCH_SEMAPHORE: tokio::sync::Semaphore =
    tokio::sync::Semaphore::const_new(MAX_CONCURRENT_PREFETCH);

/// Peek the next `PREFETCH_LOOKAHEAD` upcoming queue tracks and spawn a
/// background download for each one not already cached. Each download
/// goes into the player's L1/L2 cache via `Player::prefetch_into_cache`
/// so the track later plays via `play_data` (a cache hit) and is gapless
/// eligible. Concurrency is bounded by `PREFETCH_SEMAPHORE`.
pub(super) async fn kick_prefetch(runtime: &Runtime) {
    // Offline: the prefetch is a pure NETWORK warmer (offline-cached tracks
    // play through the offline tier without it), so skip entirely — every
    // attempt would just bounce off the API offline gate and spam the log.
    if crate::offline_mode::engine().is_offline() {
        return;
    }
    // The tier to prefetch at, resolved ONCE so the throttle estimate and
    // the actual requests below can never disagree. Local playback: the
    // device-capped resolve (#638 fix 3).
    let quality = local_playback_quality().0;
    // Adaptive throttle (#591): when the live stream is starving — panic mode
    // after a decoder underrun, or bandwidth headroom below the surviving
    // ratio — prefetch must get out of the pipe entirely. Cap 0 means "no
    // prefetch right now"; the semaphore still bounds concurrency otherwise.
    {
        use qbz_audio::network_throttle::{self, PlaybackQualityTag};
        // The bandwidth estimate describes the tier prefetch actually
        // requests below (#638 fix 3).
        let tag = match quality {
            Quality::UltraHiRes => PlaybackQualityTag::UltraHiRes,
            Quality::HiRes => PlaybackQualityTag::HiRes,
            Quality::Lossless => PlaybackQualityTag::Lossless,
            Quality::Mp3 => PlaybackQualityTag::Lossy,
        };
        let cap = network_throttle::state().current_prefetch_cap(
            network_throttle::playback_mbps_for_quality(tag),
            MAX_CONCURRENT_PREFETCH,
        );
        if cap == 0 {
            log::debug!("[qbz-slint] prefetch: skipped (network throttle cap 0)");
            return;
        }
    }
    let upcoming = runtime.core().peek_upcoming(PREFETCH_LOOKAHEAD).await;
    if upcoming.is_empty() {
        return;
    }
    for track in upcoming {
        let track_id = track.id;
        // Local tracks never need a Qobuz prefetch.
        if track.is_local {
            continue;
        }
        let player = runtime.core().player();
        if player.is_track_cached(track_id) {
            continue;
        }
        let runtime = runtime.clone();
        tokio::spawn(async move {
            let _permit = match PREFETCH_SEMAPHORE.acquire().await {
                Ok(permit) => permit,
                Err(_) => return,
            };
            let client_lock = runtime.core().client();
            let guard = client_lock.read().await;
            let Some(client) = guard.as_ref() else {
                return;
            };
            let player = runtime.core().player();
            if let Err(e) = player
                .prefetch_into_cache(client, track_id, quality)
                .await
            {
                log::debug!("[qbz-slint] prefetch: track {track_id} failed: {e}");
            }
        });
    }
}
