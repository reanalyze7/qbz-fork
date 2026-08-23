//! Seek-bar seed helper used by the session-restore path.

use super::quality::{fmt_elapsed, fmt_remaining};
use crate::{AppWindow, NowPlayingState};

/// Seed the seek bar + timers on `NowPlayingState` to a fixed position (UI
/// thread). Used by the session restore so the bar shows the resume point
/// immediately — `refresh_now_playing_meta` resets these to 0, and the poll
/// loop only catches up once playback actually starts.
pub(crate) fn seed_seek_display(w: &AppWindow, position_secs: u64, duration_secs: u64) {
    let np = w.global::<NowPlayingState>();
    let progress = if duration_secs > 0 {
        (position_secs as f32 / duration_secs as f32).clamp(0.0, 1.0)
    } else {
        0.0
    };
    np.set_duration_secs(duration_secs as i32);
    np.set_position_secs(position_secs as i32);
    np.set_progress(progress);
    np.set_elapsed(fmt_elapsed(position_secs).into());
    np.set_remaining(fmt_remaining(position_secs, duration_secs).into());
}
