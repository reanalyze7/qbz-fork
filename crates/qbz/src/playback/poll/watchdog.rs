//! Stuck-load recovery: clears the fetch spinner once real audio is
//! advancing, or force-recovers a play the engine accepted but that never
//! started producing audio.

use super::super::advance::advance_to_playable;
use super::super::engine::after_track_change;
use super::super::loading::clear_loading;
use super::super::quality::now_ms;
use super::super::state::{refresh_sidebar, UNAVAILABLE_SKIPS, PENDING_PLAY_AT_MS, PENDING_PLAY_ID, LOADING_WATCHDOG_MS};
use super::super::Runtime;
use crate::AppWindow;

/// Watchdog-driven recovery for a track whose initial load HUNG rather than
/// erroring — a transient network stall during the CMAF setup / initial-buffer
/// wait, which the legacy fallback (an error-stage path) never catches, so the
/// track would otherwise silently die at the 45s watchdog. Keyed to the stuck
/// track id: a new stuck track resets the count. Re-start the current track a
/// couple of times (the stall is usually over by the 45s mark), then skip.
static WATCHDOG_RECOVER_TRACK: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static WATCHDOG_RECOVERIES: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
const MAX_WATCHDOG_RECOVERIES: u32 = 2;

/// Clear the fetch spinner once the audio for the in-flight play is
/// actually advancing: a non-zero track with the clock moving
/// (`position > 0`) is unambiguous proof the requested track started
/// (is_playing alone can flip true transiently before the sink emits
/// the id). Keyed to PENDING_PLAY_ID so a superseded fetch doesn't
/// wipe a newer play's spinner; the keyed clear is a no-op if the
/// current audio is a different (already-cleared) id. Otherwise, force-clear
/// after the generous ceiling and recover.
pub(super) fn handle(runtime: &Runtime, weak: &slint::Weak<AppWindow>, track_id: u64, is_playing: bool, position: u64) {
    if track_id != 0 && is_playing && position > 0 {
        clear_loading(weak, track_id);
        // Real audio ends any unavailable-skip streak (Tauri parity:
        // `consecutiveSkips = 0` on successful play).
        UNAVAILABLE_SKIPS.store(0, std::sync::atomic::Ordering::SeqCst);
        // ...and any watchdog-recovery streak: the load that was stuck
        // is now producing audio, so future stalls start fresh.
        WATCHDOG_RECOVERIES.store(0, std::sync::atomic::Ordering::SeqCst);
        WATCHDOG_RECOVER_TRACK.store(0, std::sync::atomic::Ordering::SeqCst);
        return;
    }
    let pending = PENDING_PLAY_ID.load(std::sync::atomic::Ordering::Relaxed);
    if pending == 0
        || now_ms().saturating_sub(PENDING_PLAY_AT_MS.load(std::sync::atomic::Ordering::Relaxed))
            <= LOADING_WATCHDOG_MS
    {
        return;
    }
    // A load that hung (not errored) never triggers the legacy
    // fallback, so recover here instead of just clearing the
    // spinner: re-start the stuck track a couple of times, then
    // skip to the next playable one (owner: recover, worst case
    // skip). Per-track count so a new stuck track starts fresh.
    if WATCHDOG_RECOVER_TRACK.swap(pending, std::sync::atomic::Ordering::SeqCst) != pending {
        WATCHDOG_RECOVERIES.store(0, std::sync::atomic::Ordering::SeqCst);
    }
    let n = WATCHDOG_RECOVERIES.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
    // Re-arm the watchdog window so it gives the retry/skip a
    // fresh ceiling before firing again.
    PENDING_PLAY_AT_MS.store(now_ms(), std::sync::atomic::Ordering::Relaxed);
    let runtime = runtime.clone();
    let weak = weak.clone();
    if n <= MAX_WATCHDOG_RECOVERIES {
        log::warn!(
            "[qbz-slint] loading watchdog: track {pending} stuck after {}ms — recovery attempt {n}/{MAX_WATCHDOG_RECOVERIES} (re-starting current track)",
            LOADING_WATCHDOG_MS
        );
        // A fresh play generation supersedes the hung load and
        // re-attempts; the stall has usually cleared by now.
        tokio::spawn(async move {
            after_track_change(&runtime, &weak, pending).await;
        });
    } else {
        log::warn!(
            "[qbz-slint] loading watchdog: track {pending} still stuck after {MAX_WATCHDOG_RECOVERIES} retries — skipping to the next playable track"
        );
        WATCHDOG_RECOVERIES.store(0, std::sync::atomic::Ordering::SeqCst);
        WATCHDOG_RECOVER_TRACK.store(0, std::sync::atomic::Ordering::SeqCst);
        tokio::spawn(async move {
            if let Some(track) = advance_to_playable(&runtime, &weak, true).await {
                let track_id = track.id;
                after_track_change(&runtime, &weak, track_id).await;
                refresh_sidebar(true);
            } else {
                clear_loading(&weak, 0);
            }
        });
    }
}
