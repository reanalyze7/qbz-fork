//! Seamless gapless-transition detection: when the audio engine performs a
//! gapless handoff the track changes WITHOUT a stop, so this reconciles the
//! core queue pointer + now-playing card to whatever is ACTUALLY playing.

use super::state::PollLoopState;
use super::super::engine::kick_prefetch;
use super::super::meta::{record_recent, refresh_now_playing_meta};
use super::super::state::refresh_sidebar;
use super::super::Runtime;
use crate::AppWindow;

/// Detects the "track id changed while still playing" edge and, if it is
/// real, resyncs the queue pointer + metadata. Returns `true` when the tick
/// was fully handled by this phase — the caller must `continue` to the next
/// tick without running any later phase.
pub(super) async fn maybe_handle(
    runtime: &Runtime,
    weak: &slint::Weak<AppWindow>,
    state: &mut PollLoopState,
    track_id: u64,
    position: u64,
    is_playing: bool,
) -> bool {
    // When the audio engine performs a gapless handoff the track
    // changes WITHOUT a stop: `track_id` becomes the previously
    // gapless-queued id while `is_playing` stays true. Detect that
    // edge — a track-id change while still playing, where the new
    // id is not the end-of-track edge — and sync the core queue
    // pointer + refresh metadata WITHOUT calling the audible play
    // path (the player is already playing it).
    let seamless_change = track_id != 0
        && state.last_track_id != 0
        && track_id != state.last_track_id
        && is_playing
        && state.was_playing;
    if !seamless_change {
        return false;
    }
    // The audio engine advanced to `track_id` on its own — EITHER
    // a real gapless hand-off (it started the prefetched next
    // track) OR a manual new-track play that just replaced the
    // queue. Rather than guess which (the old peek-based heuristic
    // missed cases and left the card stale while the seek bar kept
    // moving — the reported populate bug), reconcile the queue
    // pointer + the now-playing card to whatever is ACTUALLY
    // playing. `sync_current_to_id` moves the pointer only when it
    // lags (a real advance); a manual play already moved it, so it
    // reports `moved == false` and we skip the double bookkeeping.
    if let Some((_, moved)) = runtime.core().sync_current_to_id(track_id).await {
        // Always refresh so title/art/meta match the live track.
        refresh_now_playing_meta(runtime, weak).await;
        // Pair the sidebar NOW PLAYING repaint with the NPB repaint
        // UNCONDITIONALLY: the sidebar's QueueState.now-playing is a
        // persistent property that otherwise holds a prior queue's
        // track when the pointer was already aligned (moved==false,
        // e.g. a manual play that set the queue before the audio
        // surfaced the new id). `false` avoids a per-transition fav
        // network pull. record_recent/kick_prefetch stay moved-gated
        // below (they must not double-fire on a non-move).
        refresh_sidebar(false);
        if moved {
            log::info!(
                "[qbz-slint] [GAPLESS] seamless transition {} -> {track_id}",
                state.last_track_id
            );
            record_recent(runtime).await;
            refresh_sidebar(true);
            // Prefetch the successors of the now-current track.
            kick_prefetch(runtime).await;
        }
        state.gapless_requested_for = 0;
    }
    // Resync the edge trackers either way so this change is not
    // re-detected on the next tick.
    state.last_track_id = track_id;
    state.seen_position = position;
    state.was_playing = is_playing;
    true
}
