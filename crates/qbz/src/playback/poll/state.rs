//! Loop-local state threaded through each tick's phases (NOT statics — see
//! the module doc on why `gapless_requested_for` must stay loop-local).

/// Per-invocation state of the poll loop (there is only ever one instance,
/// gated by `STARTED` in `start_poll_loop`).
pub(super) struct PollLoopState {
    /// Track whether the last poll observed an active track, so the
    /// end-of-track edge is detected once rather than every tick.
    pub(super) last_track_id: u64,
    pub(super) was_playing: bool,
    pub(super) seen_position: u64,
    /// Throttle for the periodic session-position save (every ~11 ticks ≈ 5s).
    pub(super) save_pos_tick: u64,
    /// Track id we have already fired a gapless prefetch for, so the
    /// 450ms ticker does not re-request it every tick.
    pub(super) gapless_requested_for: u64,
    /// Dirty-guard for the per-tick UI push. Slint Property::set has no
    /// equality check, so re-pushing identical values every 450ms dirties
    /// bindings and forces a full-window repaint even when fully idle. The
    /// snapshot holds everything its push closure depends on (f32s as bits);
    /// when unchanged, the upgrade_in_event_loop is skipped.
    pub(super) last_ui_push: Option<(u64, u64, u64, bool, u32, u32, u32, u32, u32)>,
}

impl PollLoopState {
    pub(super) fn new() -> Self {
        Self {
            last_track_id: 0,
            was_playing: false,
            seen_position: 0,
            save_pos_tick: 0,
            gapless_requested_for: 0,
            last_ui_push: None,
        }
    }
}
