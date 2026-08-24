//! End-of-track: stop-after-this-song, auto-advance, and the queue-finished
//! fallback chain.
use slint::ComponentHandle;

use super::state::PollLoopState;
use super::super::advance::{advance_to_playable, try_infinite_refill};
use super::super::engine::after_track_change;
use super::super::loading::clear_loading;
use super::super::quality::set_viz_paused;
use super::super::state::refresh_sidebar;
use super::super::Runtime;
use crate::{AppWindow, NowPlayingState};

/// Auto-advance on track end. Offline, unavailable tracks are skipped
/// (bounded — see `advance_to_playable`); exhaustion lands in the
/// queue-finished arm below. No-op unless `track_ended` (the caller's
/// end-of-track edge detection).
pub(super) async fn handle(
    runtime: &Runtime,
    weak: &slint::Weak<AppWindow>,
    state: &mut PollLoopState,
    track_ended: bool,
) {
    if !track_ended {
        return;
    }
    // Seed for InfiniteRadio: the track that just ended is still the
    // current one (advance hasn't moved the cursor yet).
    let ended_track_id = runtime.core().current_track().await.map(|t| t.id).unwrap_or(0);
    // Stop-after-this-song: if the track that just ended is the marked
    // one, HALT here (pause) — do NOT advance, do NOT infinite-refill.
    // The queue stays intact and the finished track stays parked in
    // now-playing. `consume_stop_after_if` is one-shot (clears the
    // marker). Mirrors the Tauri end-of-track `consumeStopAfterIf` ->
    // stopPlayback + early-return, ahead of any repeat/shuffle.
    if ended_track_id != 0 && runtime.core().consume_stop_after_if(ended_track_id).await {
        if let Err(e) = runtime.core().pause() {
            log::warn!("[qbz-slint] stop-after: pause failed: {e}");
        }
        // Stop counts as paused for the visualizer tap.
        set_viz_paused(runtime, true);
        state.last_track_id = 0;
        state.was_playing = false;
        state.seen_position = 0;
        state.gapless_requested_for = 0;
        refresh_sidebar(true);
        let _ = weak.upgrade_in_event_loop(|w| {
            w.global::<NowPlayingState>().set_playing(false);
        });
        return;
    }
    state.last_track_id = 0;
    state.was_playing = false;
    state.seen_position = 0;
    state.gapless_requested_for = 0;
    if let Some(track) = advance_to_playable(runtime, weak, true).await {
        let next_id = track.id;
        after_track_change(runtime, weak, next_id).await;
        refresh_sidebar(true);
    } else if try_infinite_refill(runtime, weak, ended_track_id).await {
        // Dead branch since the qbz-radio removal: try_infinite_refill
        // always returns false now (see its doc comment). Left in place
        // so the queue-finished fallback below stays the single exit.
    } else {
        log::info!("[qbz-slint] playback: queue finished");
        // Nothing more will play — force-clear any lingering spinner
        // and park the visualizer producer (stop counts as paused).
        clear_loading(weak, 0);
        set_viz_paused(runtime, true);
        let _ = weak.upgrade_in_event_loop(|w| {
            let np = w.global::<NowPlayingState>();
            np.set_playing(false);
            np.set_progress(0.0);
            np.set_position_secs(0);
        });
    }
}
