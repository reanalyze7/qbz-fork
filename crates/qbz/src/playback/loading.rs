//! Fetch-spinner bookkeeping around `PENDING_PLAY_ID`/`PENDING_PLAY_AT_MS` —
//! small but conceptually separate from the watchdog recovery logic in the
//! poll loop, which reads/writes the same statics.

use super::quality::now_ms;
use super::state::{PENDING_PLAY_AT_MS, PENDING_PLAY_ID};
use crate::{AppWindow, NowPlayingState};

/// Mark `track_id` as the in-flight play and raise the now-playing "loading"
/// flag (drives the fetch spinner on the bar, the active track row, and the
/// album play button). Source-agnostic — covers the Qobuz tier-walk and slow
/// local reads.
pub(super) fn set_loading(weak: &slint::Weak<AppWindow>, track_id: u64) {
    PENDING_PLAY_ID.store(track_id, std::sync::atomic::Ordering::Relaxed);
    PENDING_PLAY_AT_MS.store(now_ms(), std::sync::atomic::Ordering::Relaxed);
    let _ = weak.upgrade_in_event_loop(|w| {
        w.global::<NowPlayingState>().set_loading(true);
    });
}

/// Clear the loading flag if (and only if) the in-flight play is still
/// `track_id` — so a fetch that has been superseded by a newer play does not
/// wipe the newer play's spinner. Pass `0` to force-clear unconditionally
/// (queue finished / hard stop).
pub(super) fn clear_loading(weak: &slint::Weak<AppWindow>, track_id: u64) {
    if track_id != 0 && PENDING_PLAY_ID.load(std::sync::atomic::Ordering::Relaxed) != track_id {
        return;
    }
    PENDING_PLAY_ID.store(0, std::sync::atomic::Ordering::Relaxed);
    let _ = weak.upgrade_in_event_loop(|w| {
        w.global::<NowPlayingState>().set_loading(false);
    });
}
