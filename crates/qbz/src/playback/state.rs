//! Process-wide statics shared across multiple playback sub-modules, plus
//! the two functions that gate on `QUEUE_CONTROLLER`. Anything read/written
//! from a single sub-module only stays local to that sub-module instead.

use crate::queue::QueueController;

/// The Queue sidebar controller, published once the shell is up so the
/// playback paths (album/track play, skip, auto-advance) can refresh the
/// sidebar after every queue mutation.
static QUEUE_CONTROLLER: std::sync::OnceLock<QueueController> = std::sync::OnceLock::new();

/// Register the Queue sidebar controller. Called once during shell setup.
pub fn set_queue_controller(controller: QueueController) {
    let _ = QUEUE_CONTROLLER.set(controller);
}

/// The registered Queue sidebar controller, if the shell has set one up yet.
/// Used directly (rather than through `refresh_sidebar`) by callers that need
/// the controller itself — e.g. `record_recent`'s Home-rails-stale nudge.
pub(super) fn queue_controller() -> Option<&'static QueueController> {
    QUEUE_CONTROLLER.get()
}

/// Refresh the Queue sidebar from the current core queue state. No-op
/// before the controller is registered. `with_favorites` re-pulls the
/// favorite-track cache as well (used after a fresh play starts).
pub(crate) fn refresh_sidebar(with_favorites: bool) {
    if let Some(controller) = QUEUE_CONTROLLER.get() {
        if with_favorites {
            controller.refresh_with_favorites();
        } else {
            controller.refresh();
        }
    }
}

/// The track id whose audible fetch/resolve is currently in flight (the
/// "loading" track). Set the instant a play is initiated (top of
/// `play_audible`, before the multi-second Qobuz/local resolve) and
/// read by the poll loop to clear the spinner once THAT track's audio is
/// actually advancing. A NEW play overwrites it, so a superseded fetch never
/// keeps the spinner up for the wrong track. `0` = nothing loading.
pub(super) static PENDING_PLAY_ID: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

/// Epoch-millis when the in-flight play was initiated — the poll-loop watchdog
/// force-clears the spinner if audio never starts within `LOADING_WATCHDOG_MS`
/// (a play the engine accepted but that silently never advances — e.g. an
/// undecodable-but-valid-looking file — would otherwise spin forever).
pub(super) static PENDING_PLAY_AT_MS: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

/// Generous ceiling: a real fetch (even a large hi-res whole-file
/// download on a slow LAN) starts audio well under this; only a silently-stuck
/// play crosses it.
pub(super) const LOADING_WATCHDOG_MS: u64 = 45_000;

/// Consecutive auto-skips over tracks whose play failed with a TERMINAL
/// "unavailable" error (Tauri #467 parity: the Svelte playbackService kept
/// `consecutiveSkips` capped at `MAX_CONSECUTIVE_SKIPS = 5`). Reset by the
/// poll loop the moment any track actually starts producing audio.
pub(super) static UNAVAILABLE_SKIPS: std::sync::atomic::AtomicU32 =
    std::sync::atomic::AtomicU32::new(0);

/// Force-flag for the poll loop's per-tick dirty-guard (`last_ui_push`
/// in `start_poll_loop`). `refresh_now_playing_meta`
/// seeds the bar OPTIMISTICALLY (position 0 / playing true) before audio
/// actually starts; when the play is then refused or
/// fails (offline refusal, fetch error) the engine snapshot never moves, so
/// the guard would skip the corrective push forever and the bar would stick
/// on "playing". Set after every optimistic seed; the loop consumes it at the
/// top of the next tick and re-pushes engine truth unconditionally.
pub(super) static FORCE_UI_REPUSH: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Catalog-MAX stream params of the current track, cached at every
/// track-change meta push so the poll loop can compare the DELIVERED stream
/// (PlaybackEvent.sample_rate / bit_depth) against the track's advertised
/// max without an async queue read per tick (#590 follow-up; since #638
/// fix 1 the badge's main line reports the DELIVERED quality while
/// downgraded and the catalog max moves to the tooltip's "Source" line).
/// Rate in Hz (same normalization as NowPlayingState.sample-rate-hz); 0 =
/// unknown. Bits: 1 = DSD (nominal 1-bit — exempt from the bit comparison).
pub(super) static TRACK_MAX_RATE_HZ: std::sync::atomic::AtomicU32 =
    std::sync::atomic::AtomicU32::new(0);
pub(super) static TRACK_MAX_BITS: std::sync::atomic::AtomicU32 =
    std::sync::atomic::AtomicU32::new(0);

/// REQUESTED tier of the current track's stream (Qobuz format id; 0 = the
/// track is not governed by the streaming-quality preference — local and
/// ephemeral sources) plus the request-time cause (a `QualityLimit`
/// discriminant). Seeded once per track change in `refresh_now_playing_meta`
/// beside the TRACK_MAX_* stores — NEVER re-resolved per 450 ms poll tick
/// (`ui_prefs::load()` is a whole-file disk read + JSON parse). The poll
/// loop combines them with the DELIVERED stream params to name WHY a
/// downgrade happened (#638 fix 1).
pub(super) static REQUESTED_QUALITY_ID: std::sync::atomic::AtomicU32 =
    std::sync::atomic::AtomicU32::new(0);
pub(super) static REQUESTED_CAUSE: std::sync::atomic::AtomicI32 =
    std::sync::atomic::AtomicI32::new(0);
