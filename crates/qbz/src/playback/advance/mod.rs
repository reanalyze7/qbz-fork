//! Advance / auto-skip / offline gating: the "can we actually play this
//! track right now" logic used by both manual play and auto-advance.

use qbz_models::QueueTrack;

use super::Runtime;
use crate::AppWindow;

mod offline;
mod skip;

pub(super) use offline::{
    offline_playability, offline_track_playable, OfflinePlayability,
};
pub(super) use skip::{auto_skip_unavailable, is_forbidden_backoff, is_terminal_unavailable};

/// Maximum consecutive offline-unavailable tracks the queue walk skips
/// before giving up (Tauri #467 parity: `MAX_OFFLINE_SKIPS = 5`).
const MAX_OFFLINE_SKIPS: usize = 5;

/// Move the queue cursor forward/backward to the next playable track.
/// Online this returns the immediate neighbor on the first iteration unless
/// that neighbor is a LOCAL file whose path is gone (unmounted drive) — the
/// only possible online skip. Offline it also skips unavailable tracks.
/// Bounded at [`MAX_OFFLINE_SKIPS`] consecutive (Tauri #467 parity); on
/// exhaustion (bound hit, or queue edge after at least one skip) playback
/// stops and ONE toast reports it — worded for the drive when every skip was
/// a missing local file, for offline otherwise.
///
/// The gapless-prefetched target never passes through here: a gapless
/// hand-off happens inside the audio engine and surfaces to the poll loop
/// as a seamless track-id change (no advance call), so the "never skip the
/// gapless target" exemption is structural.
pub(super) async fn advance_to_playable(
    runtime: &Runtime,
    weak: &slint::Weak<AppWindow>,
    forward: bool,
) -> Option<QueueTrack> {
    let mut skips = 0usize;
    let mut missing_files = 0usize;
    // One message for the whole walk: when every skipped track was a local
    // file that isn't on disk, point at the drive; any other mix keeps the
    // offline wording (online, FileMissing is the only possible skip).
    let walk_toast = |skips: usize, missing_files: usize| {
        if missing_files == skips {
            qbz_i18n::t("Files not available — is the drive mounted?")
        } else {
            qbz_i18n::t("No tracks available offline")
        }
    };
    loop {
        let step = if forward {
            runtime.core().next_track().await
        } else {
            runtime.core().previous_track().await
        };
        let Some(track) = step else {
            // Queue edge. Quiet when nothing was skipped (the normal end of
            // queue); one toast when the walk dropped tracks on the way.
            if skips > 0 {
                crate::toast::show_weak(
                    weak,
                    walk_toast(skips, missing_files),
                    crate::ToastKind::Warning,
                );
            }
            return None;
        };
        match offline_playability(&track) {
            OfflinePlayability::Playable => return Some(track),
            OfflinePlayability::FileMissing => missing_files += 1,
            _ => {}
        }
        skips += 1;
        log::info!(
            "[qbz-slint] advance: skipping unavailable track {} ({skips}/{MAX_OFFLINE_SKIPS})",
            track.id
        );
        if skips >= MAX_OFFLINE_SKIPS {
            if let Err(e) = runtime.core().stop() {
                log::warn!("[qbz-slint] advance: stop after skip bound failed: {e}");
            }
            crate::toast::show_weak(
                weak,
                walk_toast(skips, missing_files),
                crate::ToastKind::Warning,
            );
            return None;
        }
    }
}

/// When the queue is exhausted and `InfiniteRadio` autoplay is on, this used
/// to build a smart artist radio (seeded by the just-finished track) via the
/// local `qbz-radio` pool builder and start it, replacing the spent queue.
///
/// `qbz-radio` was removed (REMOVAL-SPEC.md §6 "Radio" — the Qobuz-generated
/// stations feature); this was its only OTHER caller, discovered while
/// removing the crate. There is no remaining refill mechanism, so this now
/// always returns `false` (never refills) — `InfiniteRadio` autoplay silently
/// stops advancing at the end of the queue instead of crashing. The
/// `AutoplayMode::InfiniteRadio` setting/UI is untouched (out of scope here);
/// flagged in the removal report as a functional regression to resolve
/// separately (e.g. drop the mode, or reseed the queue some other way).
///
/// DO NOT delete this "dead" call site in the track-end handler — it is
/// intentionally kept as the single fallback-chain exit point should a
/// refill mechanism return one day.
pub(super) async fn try_infinite_refill(
    _runtime: &Runtime,
    _weak: &slint::Weak<AppWindow>,
    _seed_track_id: u64,
) -> bool {
    false
}
