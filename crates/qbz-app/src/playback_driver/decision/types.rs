use qbz_player::PlaybackEvent;

/// A side effect the shell must perform this tick. Produced by [`super::plan_tick`],
/// executed by [`crate::playback_driver::run_driver`]. Consumed by later tasks (T7 next/prev reuses the
/// advance ritual, T10 wires `ReportEdge`, T11 the settings reload).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DriverAction {
    /// Reconcile the queue cursor to the id the engine is actually playing
    /// (a seamless gapless hand-off; `playback.rs:4340`).
    SyncCursorTo(u64),
    /// Pre-queue this upcoming track's bytes for a gapless transition
    /// (`playback.rs:4387`).
    ArmGapless(u64),
    /// The current track ended and there is a next playable track — run the
    /// full advance ritual (`playback.rs:4743`).
    AdvanceAndPlay,
    /// The ended track was stop-after-marked — halt (pause), never advance
    /// (`playback.rs:4720`).
    PauseStopAfter,
    /// Persist the live position (throttled ~5 s; `playback.rs:4307`).
    SavePosition(u64),
    /// Latch a drained stream-error message so `status` stays diagnosable
    /// (`playback.rs:4111`).
    LatchError(String),
    /// Emit an outbound QConnect renderer-state report on a transition or the
    /// ~2 s periodic cadence (`playback.rs:4648`).
    ReportEdge,
    /// The current track ended and nothing is playable — stop
    /// (`playback.rs:4751`).
    QueueFinished,
}

/// The previous tick's snapshot: the desktop loop's `last_track_id` /
/// `seen_position` / `was_playing` (plus duration for [`DriverState::after`]).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LastTick {
    pub track_id: u64,
    pub position: u64,
    pub duration: u64,
    pub is_playing: bool,
}

impl LastTick {
    pub(super) fn from_event(ev: &PlaybackEvent) -> Self {
        Self {
            track_id: ev.track_id,
            position: ev.position,
            duration: ev.duration,
            is_playing: ev.is_playing,
        }
    }
}

/// The driver's carried-over state between ticks — the loop-local `mut`
/// variables of `start_poll_loop`, hoisted into a value so the decision is a
/// pure function of it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DriverState {
    /// Previous tick (`last_track_id`, `seen_position`, `was_playing`).
    pub last: LastTick,
    /// ~11-tick throttle counter for the periodic position save.
    pub save_pos_tick: u64,
    /// Track id an `ArmGapless` already fired for, so the ticker does not
    /// re-request it every tick (`gapless_requested_for`).
    pub gapless_requested_for: u64,
    /// ~4-tick throttle counter for the periodic QConnect report.
    pub report_tick: u64,
    /// Last track id we emitted a `ReportEdge` for (`last_reported_track_id`).
    pub last_reported_track_id: u64,
    /// Last play-state we emitted a `ReportEdge` for (`last_reported_playing`).
    pub last_reported_playing: bool,
}

impl DriverState {
    /// A state whose `last` snapshot (and report trackers) come from `ev` — the
    /// "the previous tick looked like this" constructor used by the tests and by
    /// the shell to seed a baseline.
    pub fn after(ev: &PlaybackEvent) -> DriverState {
        DriverState {
            last: LastTick::from_event(ev),
            save_pos_tick: 0,
            gapless_requested_for: 0,
            report_tick: 0,
            last_reported_track_id: ev.track_id,
            last_reported_playing: ev.is_playing,
        }
    }
}
