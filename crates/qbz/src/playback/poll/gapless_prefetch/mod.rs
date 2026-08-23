//! Gapless prefetch trigger: when the engine signals it wants the next
//! track pre-queued (`gapless_ready`) and nothing is queued yet, resolve
//! the next upcoming queue track and hand its bytes/path to the engine's
//! gapless queue. The `gapless_requested_for` guard (loop-local state, NOT
//! a static) stops the 450ms ticker from re-firing while the fetch is in
//! flight.

mod local;
mod network;

use super::state::PollLoopState;
use super::super::advance::offline_track_playable;
use super::super::Runtime;
use crate::AppWindow;

/// Check the gapless-ready gate and, if a suitable upcoming track exists,
/// spawn its pre-queue fetch (network or local, per its source).
pub(super) async fn maybe_trigger(
    runtime: &Runtime,
    weak: &slint::Weak<AppWindow>,
    state: &mut PollLoopState,
    track_id: u64,
    gapless_ready: bool,
    gapless_next_track_id: u64,
) {
    // Stop-after guard (last condition, short-circuited): if the CURRENT
    // track is marked "stop after this", suppress the gapless pre-queue so
    // it ends naturally and the track-end handler can fire
    // `consume_stop_after_if`. Mirrors the Tauri `setGaplessGetNextTrackId`
    // null-return for the marked track. Without this the engine seamlessly
    // hands off and the marker never fires.
    if !(gapless_ready
        && gapless_next_track_id == 0
        && track_id != 0
        && state.gapless_requested_for != track_id
        && runtime.core().get_stop_after().await != Some(track_id))
    {
        return;
    }
    let upcoming = runtime.core().peek_upcoming(1).await;
    let Some(next) = upcoming.into_iter().next() else {
        return;
    };
    // Never queue the current track as its own next. Offline,
    // an unavailable successor is not pre-queued either (the
    // same playable rule as the advance walk) — the track-end
    // auto-advance then skips it properly instead of the
    // engine gapless-handing into a refused fetch.
    if next.id != track_id && !next.is_local && offline_track_playable(&next) {
        state.gapless_requested_for = track_id;
        network::spawn_fetch(runtime, weak, next.id);
    } else if next.id != track_id && next.is_local {
        state.gapless_requested_for = track_id;
        local::spawn_fetch(runtime, next.id);
    }
}
