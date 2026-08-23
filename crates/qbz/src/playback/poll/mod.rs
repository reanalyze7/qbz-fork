//! The playback poll loop: the ONLY source of playback events (no event
//! stream from the player), driving the now-playing bar, MPRIS/tray/
//! notifications, gapless prefetch, seamless-transition reconciliation,
//! watchdog recovery, session-position persistence, and auto-advance.

mod gapless_prefetch;
mod seamless;
mod state;
mod stream_errors;
mod track_end;
mod tray_edge;
mod ui_push;
mod watchdog;

use state::PollLoopState;
use super::state::FORCE_UI_REPUSH;
use super::Runtime;
use crate::AppWindow;

/// Start the playback poll loop. Runs for the app lifetime: every ~450ms
/// it reads the player event and pushes position / progress onto
/// `NowPlayingState`. When a track ends it auto-advances the queue.
///
/// Guarded against double-start: the shell can now be entered twice per
/// process (offline session, then the D2 recovery login runs the full
/// online entry over it) and a second loop would double the track-end
/// auto-advance.
pub fn start_poll_loop(runtime: Runtime, weak: slint::Weak<AppWindow>, handle: tokio::runtime::Handle) {
    static STARTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if STARTED.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    let spawn_handle = handle.clone();
    spawn_handle.spawn(async move {
        let mut state = PollLoopState::new();
        let mut ticker = tokio::time::interval(std::time::Duration::from_millis(450));
        loop {
            ticker.tick().await;
            tick(&runtime, &weak, &mut state).await;
        }
    });
}

/// One 450ms tick: run every phase in sequence.
async fn tick(runtime: &Runtime, weak: &slint::Weak<AppWindow>, state: &mut PollLoopState) {
    // A meta refresh outside this loop just seeded the bar
    // optimistically (position 0 / playing true — see FORCE_UI_REPUSH).
    // Drop the dirty-guard so this tick re-pushes engine truth
    // even when the raw snapshot did not move (refused/failed play,
    // paused track hit by a mid-track quality patch).
    if FORCE_UI_REPUSH.swap(false, std::sync::atomic::Ordering::Relaxed) {
        state.last_ui_push = None;
    }

    stream_errors::surface(runtime, weak);

    let event = runtime.core().player().get_playback_event();
    let track_id = event.track_id;
    let position = event.position;
    let duration = event.duration;
    let is_playing = event.is_playing;
    let volume = event.volume;
    // DELIVERED stream params (what the engine actually decodes after
    // the streaming-quality downgrade, #590). 0 = unknown / no stream.
    let eff_rate_hz = event.sample_rate.unwrap_or(0);
    let eff_bits = event.bit_depth.unwrap_or(0);
    // Persist the live position periodically (~5s) while playing so a
    // crash keeps a near-current resume point (no-op unless
    // `persist_session` is on; `position` is in seconds).
    state.save_pos_tick = state.save_pos_tick.wrapping_add(1);
    if is_playing && track_id != 0 && state.save_pos_tick % 11 == 0 {
        crate::session_persist::save_position(position);
    }
    // Streaming buffer fill, for the seek-bar cache overlay.
    let cache = event.buffer_progress.unwrap_or(0.0);
    // Seek lock: while streaming (`buffer_progress` is Some), the user
    // can only seek up to what has downloaded; fully-available tracks
    // (None) seek freely.
    let seekable_max = event.buffer_progress.map(|p| p.clamp(0.0, 1.0)).unwrap_or(1.0);

    if seamless::maybe_handle(runtime, weak, state, track_id, position, is_playing).await {
        return;
    }

    gapless_prefetch::maybe_trigger(
        runtime,
        weak,
        state,
        track_id,
        event.gapless_ready,
        event.gapless_next_track_id,
    )
    .await;

    // Detect end-of-track: there was a track, it has reached the
    // end (position within the duration) and is no longer playing.
    let track_ended = state.was_playing
        && !is_playing
        && state.last_track_id != 0
        && (track_id == 0 || track_id == state.last_track_id)
        && duration > 0
        && state.seen_position + 2 >= duration;

    ui_push::maybe_push(
        runtime, weak, state, track_id, position, duration, is_playing, volume, cache, seekable_max,
        eff_rate_hz, eff_bits,
    );

    watchdog::handle(runtime, weak, track_id, is_playing, position);

    if track_id != 0 {
        state.last_track_id = track_id;
        state.seen_position = position;
    }
    tray_edge::on_transition(is_playing, state.was_playing, position);
    state.was_playing = is_playing;

    track_end::handle(runtime, weak, state, track_ended).await;
}
