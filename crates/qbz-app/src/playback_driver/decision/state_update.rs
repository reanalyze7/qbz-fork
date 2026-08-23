use qbz_player::PlaybackEvent;

use super::types::{DriverAction, DriverState, LastTick};

/// The pure state-update rule applied after [`super::plan_tick`] each tick. Replicates
/// how the desktop loop mutates its carried variables, branch by branch.
pub fn advance_state(
    prev: &DriverState,
    ev: &PlaybackEvent,
    actions: &[DriverAction],
) -> DriverState {
    let seamless = actions
        .iter()
        .any(|a| matches!(a, DriverAction::SyncCursorTo(_)));
    let armed = actions
        .iter()
        .any(|a| matches!(a, DriverAction::ArmGapless(_)));
    let ended = actions.iter().any(|a| {
        matches!(
            a,
            DriverAction::AdvanceAndPlay
                | DriverAction::PauseStopAfter
                | DriverAction::QueueFinished
        )
    });
    let reported = actions.iter().any(|a| matches!(a, DriverAction::ReportEdge));

    // save_pos_tick advances every tick — playback.rs:4305 runs before the
    // seamless `continue`.
    let save_pos_tick = prev.save_pos_tick.wrapping_add(1);

    if seamless {
        // Seamless branch (playback.rs:4363-4369): last <- ev, gapless guard
        // cleared, report trackers untouched (the `continue` precedes the report
        // block, so report_tick does NOT advance this tick).
        return DriverState {
            last: LastTick::from_event(ev),
            save_pos_tick,
            gapless_requested_for: 0,
            report_tick: prev.report_tick,
            last_reported_track_id: prev.last_reported_track_id,
            last_reported_playing: prev.last_reported_playing,
        };
    }

    // Non-seamless: the report block runs (playback.rs:4648) so report_tick
    // advances; the report trackers move only when a ReportEdge fired.
    let report_tick = prev.report_tick.wrapping_add(1);
    let (last_reported_track_id, last_reported_playing) = if reported {
        (ev.track_id, ev.is_playing)
    } else {
        (prev.last_reported_track_id, prev.last_reported_playing)
    };

    // Edge trackers (playback.rs:4676-4700): last_track_id/seen_position update
    // only when track_id != 0; was_playing tracks is_playing unconditionally.
    let mut last = if ev.track_id != 0 {
        LastTick::from_event(ev)
    } else {
        LastTick {
            track_id: prev.last.track_id,
            position: prev.last.position,
            duration: prev.last.duration,
            is_playing: ev.is_playing,
        }
    };
    let mut gapless_requested_for = if armed {
        ev.track_id
    } else {
        prev.gapless_requested_for
    };

    // Track-end handler resets the edge trackers + the gapless guard
    // (playback.rs:4728-4742, both the stop-after and advance branches).
    if ended {
        last = LastTick::default();
        gapless_requested_for = 0;
    }

    DriverState {
        last,
        save_pos_tick,
        gapless_requested_for,
        report_tick,
        last_reported_track_id,
        last_reported_playing,
    }
}
