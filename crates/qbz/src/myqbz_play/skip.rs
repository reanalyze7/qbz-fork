//! Part B: skip-to-ITEM (boundary nav).
//!
//! Spec 40 §5.6 + §6 (`v2_skip_to_next_item` / `v2_skip_to_previous_item`):
//! jump the queue cursor to the START of the next / previous ITEM (album /
//! playlist / track group) rather than the next / previous TRACK. The boundary
//! key per track = `source_item_id_hint` (stamped by the resolver), else the
//! `album_id` fallback — both already on `QueueTrack`. The math is the PURE,
//! already-shared `qbz_mixtape::enqueue::{next_item_index, previous_item_index}`
//! (the 3-second prev rule lives there); these helpers only read the live queue
//! from `qbz-core`, call that math, and `play_index` the target.
//!
//! **These NEVER touch the global `playback::next()` / `previous()`** — the
//! normal transport stays track-by-track. They are headless entry points over
//! `qbz-core` so a future UI trigger (or QConnect / CLI) can drive them without
//! re-implementing the boundary detection.
//!
//! **UI trigger: DEFERRED.** Tauri registers both commands
//! (`src-tauri/src/lib.rs`) but has ZERO frontend callsites — there is no
//! skip-album button anywhere in the Tauri UI, so there is no faithful UI home
//! to port 1:1. Forcing a button into the shared next/prev transport would risk
//! the global transport for a behavior Tauri itself never surfaced. So the
//! helpers land headless + tested-by-shared-crate; wiring a UI trigger waits
//! until the product asks for one (then it calls these, no transport rewrite).

use crate::playback::{after_track_change, refresh_sidebar};
use crate::AppWindow;

use super::Runtime;

/// Skip to the START of the NEXT item in the live queue (spec 40 §6
/// `v2_skip_to_next_item`). Reads the current queue + cursor from `qbz-core`,
/// finds the first track whose item-boundary differs from the current item via
/// `next_item_index`, and `play_index`es it. No-op at the last item / empty
/// queue / no current cursor.
#[allow(dead_code)] // Reserved: ported (spec 40 §6), pending Mixtape item-skip wiring.
pub async fn skip_to_next_item(runtime: &Runtime, weak: &slint::Weak<AppWindow>) {
    let (queue, current) = runtime.core().get_all_queue_tracks().await;
    let Some(current) = current else {
        log::debug!("[qbz-slint] myqbz_play: skip_to_next_item — no current track");
        return;
    };
    match qbz_mixtape::enqueue::next_item_index(&queue, current) {
        Some(target) => {
            if let Some(track) = runtime.core().play_index(target).await {
                let track_id = track.id;
                after_track_change(runtime, weak, track_id).await;
                refresh_sidebar(true);
            }
        }
        None => {
            log::debug!("[qbz-slint] myqbz_play: skip_to_next_item — already at last item");
        }
    }
}

/// Skip to the START of the PREVIOUS item (or restart the current one) in the
/// live queue (spec 40 §6 `v2_skip_to_previous_item`). Reads the current queue
/// + cursor + elapsed position from `qbz-core`, applies the 3-second prev rule
/// via `previous_item_index` (elapsed > 3s OR mid-item → restart current item;
/// else jump to the previous item's start), and `play_index`es the target.
#[allow(dead_code)] // Reserved: ported (spec 40 §6), pending Mixtape item-skip wiring.
pub async fn skip_to_previous_item(runtime: &Runtime, weak: &slint::Weak<AppWindow>) {
    let (queue, current) = runtime.core().get_all_queue_tracks().await;
    let Some(current) = current else {
        log::debug!("[qbz-slint] myqbz_play: skip_to_previous_item — no current track");
        return;
    };
    // `PlaybackState.position` is in whole seconds (same unit the seek path
    // multiplies against `duration`); the boundary math wants elapsed ms.
    let elapsed_ms = runtime.core().get_playback_state().position * 1_000;
    match qbz_mixtape::enqueue::previous_item_index(&queue, current, elapsed_ms) {
        Some(target) => {
            if let Some(track) = runtime.core().play_index(target).await {
                let track_id = track.id;
                after_track_change(runtime, weak, track_id).await;
                refresh_sidebar(true);
            }
        }
        None => {
            log::debug!("[qbz-slint] myqbz_play: skip_to_previous_item — no previous item");
        }
    }
}
