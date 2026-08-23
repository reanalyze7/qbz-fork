use qbz_player::PlaybackEvent;

use super::queue::{next_playable, QueueSnapshot};
use super::types::{DriverAction, DriverState};
use crate::playback_driver::{
    MAX_OFFLINE_SKIPS, QCONNECT_REPORT_EVERY_N_TICKS, SAVE_POSITION_EVERY_N_TICKS,
};

/// The pure per-tick decision. Side-effect-free: given the carried state, the
/// live player event, the queue projection and any drained stream error, decide
/// which [`DriverAction`]s the shell must perform. Order mirrors the desktop
/// loop so the shell executes effects in the same sequence.
pub fn plan_tick(
    state: &DriverState,
    ev: &PlaybackEvent,
    queue: &QueueSnapshot,
    stream_error: Option<&str>,
) -> Vec<DriverAction> {
    let mut actions = Vec::new();
    let last = &state.last;

    // 1. Stream-error latch (playback.rs:4111): the player records a
    //    user-readable message drained exactly once per failure.
    if let Some(msg) = stream_error {
        actions.push(DriverAction::LatchError(msg.to_string()));
    }

    // 2. Periodic session-position save (playback.rs:4305-4308): ~11 ticks ≈ 5 s
    //    while a track is actually playing.
    let next_save_tick = state.save_pos_tick.wrapping_add(1);
    if ev.is_playing && ev.track_id != 0 && next_save_tick % SAVE_POSITION_EVERY_N_TICKS == 0 {
        actions.push(DriverAction::SavePosition(ev.position));
    }

    // 3. Seamless gapless transition (playback.rs:4324-4371): the engine advanced
    //    to a new track WITHOUT a stop (track-id change while still playing).
    //    Sync the cursor and STOP — the desktop `continue`s before every block
    //    below, so no other action fires this tick.
    let seamless_change = ev.track_id != 0
        && last.track_id != 0
        && ev.track_id != last.track_id
        && ev.is_playing
        && last.is_playing;
    if seamless_change {
        actions.push(DriverAction::SyncCursorTo(ev.track_id));
        return actions;
    }

    // 4. Gapless prefetch trigger (playback.rs:4387-4401): the engine wants the
    //    next track pre-queued and none is armed; arm the first playable upcoming
    //    exactly once per current track, suppressed when the current track is
    //    stop-after-marked (so it ends naturally and the marker can fire).
    if ev.gapless_ready
        && ev.gapless_next_track_id == 0
        && ev.track_id != 0
        && state.gapless_requested_for != ev.track_id
        && queue.stop_after != Some(ev.track_id)
    {
        if let Some(&(next_id, playable)) = queue.upcoming.first() {
            if next_id != ev.track_id && playable {
                actions.push(DriverAction::ArmGapless(next_id));
            }
        }
    }

    // 5. End-of-track edge (playback.rs:4489-4496): the previous tick was
    //    playing, this tick is not, the track id held (or went to 0), and the
    //    previous position was within 2 s of the (current) duration. Uses the
    //    live `ev.duration` guard + previous `last.position` exactly as the
    //    desktop's `duration > 0 && seen_position + 2 >= duration`.
    let track_ended = last.is_playing
        && !ev.is_playing
        && last.track_id != 0
        && (ev.track_id == 0 || ev.track_id == last.track_id)
        && ev.duration > 0
        && last.position + 2 >= ev.duration;

    // 6. QConnect report edge (playback.rs:4648-4673): report on a track/play
    //    transition OR the ~2 s periodic cadence while playing. Runs regardless
    //    of `track_ended` (the desktop report block precedes the advance block).
    let next_report_tick = state.report_tick.wrapping_add(1);
    if ev.track_id != 0 {
        let transition = ev.track_id != state.last_reported_track_id
            || ev.is_playing != state.last_reported_playing;
        let periodic = ev.is_playing && next_report_tick % QCONNECT_REPORT_EVERY_N_TICKS == 0;
        if transition || periodic {
            actions.push(DriverAction::ReportEdge);
        }
    }

    // 7. Track-end handling (playback.rs:4705-4762): stop-after HALTS (pause,
    //    never advance, ahead of any repeat/shuffle); else advance to the next
    //    playable track; else the queue is finished. repeat one/all always yield
    //    a next track (QueueManager owns the replay/wrap), so they never finish.
    if track_ended {
        if queue.current != 0 && queue.stop_after == Some(queue.current) {
            actions.push(DriverAction::PauseStopAfter);
        } else if queue.repeat == "one" || queue.repeat == "all" {
            actions.push(DriverAction::AdvanceAndPlay);
        } else if next_playable(&queue.upcoming, MAX_OFFLINE_SKIPS).is_some() {
            actions.push(DriverAction::AdvanceAndPlay);
        } else {
            actions.push(DriverAction::QueueFinished);
        }
    }

    actions
}
