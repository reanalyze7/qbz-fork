//! Startup queue restoration.

use std::sync::atomic::Ordering;

use qbz_models::QueueTrack;

use super::convert::{from_persisted, repeat_from_str};
use super::state::{persist_enabled, Runtime, PENDING_RESUME, PENDING_RESUME_TRACK, RESUME_POSITION, STORE};

/// Restore the persisted queue at startup. Returns true if a non-empty queue was
/// restored (so the caller refreshes the now-playing bar). Restores PAUSED; when
/// `resume_playback_position` is on, primes the pending-resume slot for Phase B.
pub async fn restore(runtime: &Runtime) -> bool {
    if !persist_enabled() {
        log::info!("[qbz-slint] session_persist: restore skipped (persist_session off)");
        return false;
    }
    let snapshot = {
        let guard = STORE.lock().unwrap();
        let Some(store) = guard.as_ref() else {
            log::warn!("[qbz-slint] session_persist: restore skipped (store not open)");
            return false;
        };
        match store.load_session() {
            Ok(s) => s,
            Err(e) => {
                log::warn!("[qbz-slint] session_persist: load failed: {e}");
                return false;
            }
        }
    };
    let pb_sess = snapshot.playback;
    if pb_sess.queue_tracks.is_empty() {
        log::info!("[qbz-slint] session_persist: nothing to restore (saved queue is empty)");
        return false;
    }
    let position = pb_sess.current_position_secs;
    let count = pb_sess.queue_tracks.len();
    let index = pb_sess.current_index;
    let tracks: Vec<QueueTrack> = pb_sess
        .queue_tracks
        .into_iter()
        .map(from_persisted)
        .collect();
    // The current track's id, so the resume position is applied ONLY when this
    // exact track is the first one played after the restore.
    let current_track_id = index.and_then(|i| tracks.get(i)).map(|t| t.id).unwrap_or(0);
    runtime
        .core()
        .set_queue_with_order(tracks, index, pb_sess.shuffle_enabled, None)
        .await;
    runtime
        .core()
        .set_repeat_mode(repeat_from_str(&pb_sess.repeat_mode))
        .await;
    // The queue session carries the authoritative last volume; apply it to the
    // player (the slider also seeds from ui_prefs, but this keeps them in step).
    let _ = runtime.core().set_volume(pb_sess.volume);
    if RESUME_POSITION.load(Ordering::Relaxed) && position > 0 && current_track_id != 0 {
        PENDING_RESUME.store(position, Ordering::Relaxed);
        PENDING_RESUME_TRACK.store(current_track_id, Ordering::Relaxed);
    }
    log::info!(
        "[qbz-slint] session_persist: restored {count} queue tracks (index {index:?}), paused; \
         resume position {position}s (consumed on first play when enabled)"
    );
    true
}
