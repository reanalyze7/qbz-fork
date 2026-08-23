//! Auto-skip over a track whose play failed, plus the error classifiers
//! that decide whether a failure is skip-worthy at all.

use super::super::engine::after_track_change;
use super::super::quality::set_viz_paused;
use super::super::state::{refresh_sidebar, UNAVAILABLE_SKIPS};
use super::super::Runtime;
use super::advance_to_playable;
use crate::{AppWindow, NowPlayingState};

/// Bound on [`UNAVAILABLE_SKIPS`] (Tauri #467 parity: the Svelte
/// playbackService kept `consecutiveSkips` capped at
/// `MAX_CONSECUTIVE_SKIPS = 5`).
const MAX_UNAVAILABLE_SKIPS: u32 = 5;

/// True when a stringified play error means the track cannot play now or
/// ever at any quality — as opposed to a transient network/server failure
/// (those are already retried with backoff inside the client and must NOT
/// cost the user a good track). The `ApiError` Display texts survive the
/// `Result<(), String>` flattening in `Player::play_track` ("Failed to get
/// stream URL: {ApiError}"), so a substring match is the same pragmatic
/// contract the Tauri frontend used (`errorStr.includes(...)`).
pub(in super::super) fn is_terminal_unavailable(e: &str) -> bool {
    e.contains("no longer available") // ApiError::TrackUnavailable
        || e.contains("not streamable") // ApiError::NonStreamable
        || e.contains("No valid quality available") // ApiError::NoQualityAvailable
}

/// True when a play error is a Qobuz 403 / the client-side 403 back-off (issue
/// #637). This is NOT the track's fault (every track 403s the same way), so it
/// must NOT count as an "unavailable" skip — skipping good tracks helps
/// nothing and, pre-fix, burned through 5 of them showing a misleading "no
/// longer available". Instead we stop cleanly and tell the user. Matches the
/// `ApiError::Forbidden` / `ApiError::ForbiddenCircuitOpen` Display texts that
/// survive the `Result<(), String>` flattening.
pub(in super::super) fn is_forbidden_backoff(e: &str) -> bool {
    e.contains("Access forbidden by Qobuz") // ApiError::Forbidden
        || e.contains("backing off after repeated 403s") // ApiError::ForbiddenCircuitOpen
}

/// Advance past a track whose play failed with a terminal "unavailable"
/// error, mirroring the Tauri frontend's `autoSkipToNext`: toast, honor
/// stop-after, bounded consecutive counter, then reuse the real advance
/// machinery. `after_track_change` re-enters `play_audible`, so this is an
/// async recursion — bounded by `MAX_UNAVAILABLE_SKIPS` (counter reset in
/// the poll loop on real audio). The signature RETURNS a boxed `dyn Future
/// + Send` instead of being an `async fn`: the recursion makes the future's
/// Send-ness self-referential, and with an inferred (`impl Future`) type the
/// compiler hits a query cycle ("cannot satisfy ...: Send"). Declaring the
/// concrete boxed type in the signature is what cuts the cycle — the same
/// shape the `async_recursion` macro expands to.
pub(in super::super) fn auto_skip_unavailable<'a>(
    runtime: &'a Runtime,
    weak: &'a slint::Weak<AppWindow>,
    failed_track_id: u64,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
    Box::pin(async move {
        crate::toast::show_weak(
            weak,
            qbz_i18n::t("This track is no longer available"),
            crate::ToastKind::Warning,
        );
        // Stop-after-this-song on the failed track: halt exactly like the
        // natural end-of-track arm would (no advance, queue intact); the
        // marker is one-shot and must be consumed here or it would leak onto
        // a track it was never armed for.
        if failed_track_id != 0 && runtime.core().consume_stop_after_if(failed_track_id).await {
            set_viz_paused(runtime, true);
            let _ = weak.upgrade_in_event_loop(|w| {
                w.global::<NowPlayingState>().set_playing(false);
            });
            return;
        }
        let skips = UNAVAILABLE_SKIPS.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
        if skips > MAX_UNAVAILABLE_SKIPS {
            log::warn!(
                "[qbz-slint] playback: {MAX_UNAVAILABLE_SKIPS} consecutive unavailable tracks — stopping the skip walk"
            );
            crate::toast::show_weak(
                weak,
                qbz_i18n::t("No available tracks to play"),
                crate::ToastKind::Warning,
            );
            set_viz_paused(runtime, true);
            let _ = weak.upgrade_in_event_loop(|w| {
                w.global::<NowPlayingState>().set_playing(false);
            });
            return;
        }
        if let Some(track) = advance_to_playable(runtime, weak, true).await {
            let next_id = track.id;
            after_track_change(runtime, weak, next_id).await;
            refresh_sidebar(true);
        }
    })
}
