use qbz_core::{FrontendAdapter, QbzCore};
use qbz_models::{Quality, QueueTrack};

use super::session::{repeat_to_str, save_session_now};
use super::MAX_OFFLINE_SKIPS;
use crate::shell::AppRuntime;

/// The FULL advance ritual, reused verbatim by the CLI next/prev routes (T7):
/// bounded skip-walk to the next (or previous) playable track → play it → warm
/// the successors for gapless → persist the session. Never a bare cursor move
/// (02 §2.2). The skip-walk mirrors `playback.rs::advance_to_playable` (capped
/// at `MAX_OFFLINE_SKIPS`); `next_track()`/`previous_track()` are the atomic
/// cursor movers — `forward` selects which one, exactly like the desktop's
/// `advance_to_playable(runtime, weak, forward)` (`crates/qbz/src/playback.rs:358`),
/// so `qbzd next` and `qbzd prev` share this one ritual instead of duplicating
/// the play → prefetch → persist tail.
pub async fn advance_and_play<A: FrontendAdapter + Send + Sync + 'static>(
    runtime: &AppRuntime<A>,
    quality: Quality,
    forward: bool,
) -> Result<Option<QueueTrack>, String> {
    let core = runtime.core();
    let mut skips = 0usize;
    let next = loop {
        let step = if forward {
            core.next_track().await
        } else {
            core.previous_track().await
        };
        let Some(track) = step else {
            break None; // queue edge
        };
        // Playable gate: local files always attempt (they play from disk);
        // streamable Qobuz tracks OK. Daemon P0 has no offline tier, so an
        // unstreamable remote track is a skip (mirrors advance_to_playable).
        if track.streamable || track.is_local {
            break Some(track);
        }
        skips += 1;
        log::info!("[qbzd] driver: skipping unavailable track {} ({skips}/{MAX_OFFLINE_SKIPS})", track.id);
        if skips >= MAX_OFFLINE_SKIPS {
            let _ = core.stop();
            break None;
        }
    };
    let Some(track) = next else {
        return Ok(None);
    };
    let track_id = track.id;
    core.play_track_resolved(track_id, quality, None, None, 0)
        .await?;
    // Warm the successors so the next transition can be gapless (best-effort).
    prefetch_successors(runtime, quality).await;
    // Persist the session (queue + current + position) so a restart resumes.
    save_session_now(runtime).await;
    Ok(Some(track))
}

/// Warm the player cache for the next upcoming track (best-effort; failures are
/// logged, never fatal). Mirrors `playback.rs::kick_prefetch` for the daemon:
/// only remote, non-local, not-already-cached tracks are fetched.
async fn prefetch_successors<A: FrontendAdapter + Send + Sync + 'static>(
    runtime: &AppRuntime<A>,
    quality: Quality,
) {
    let core = runtime.core();
    let upcoming = core.peek_upcoming(1).await;
    let Some(next) = upcoming.into_iter().next() else {
        return;
    };
    if next.is_local {
        return;
    }
    let player = core.player();
    if player.is_track_cached(next.id) {
        return;
    }
    let client_lock = core.client();
    let guard = client_lock.read().await;
    let Some(client) = guard.as_ref() else {
        return;
    };
    if let Err(e) = player.prefetch_into_cache(client, next.id, quality).await {
        log::debug!("[qbzd] driver: prefetch track {} failed: {e}", next.id);
    }
}

/// Project the live queue into the decision's `QueueSnapshot`.
pub(super) async fn queue_snapshot<A: FrontendAdapter + Send + Sync + 'static>(
    core: &QbzCore<A>,
) -> super::decision::QueueSnapshot {
    let state = core.get_queue_state_full().await;
    let current = state.current_track.as_ref().map(|t| t.id).unwrap_or(0);
    let upcoming = state
        .upcoming
        .iter()
        .map(|t| (t.id, t.streamable || t.is_local))
        .collect();
    super::decision::QueueSnapshot {
        current,
        upcoming,
        repeat: repeat_to_str(state.repeat).to_string(),
        stop_after: state.stop_after_track_id,
        // Autoplay "infinite" is a later-task wiring; P0 never sets it.
        autoplay_infinite: false,
    }
}
