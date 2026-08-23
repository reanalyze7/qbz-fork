use qbz_core::FrontendAdapter;
use qbz_models::{QueueTrack, RepeatMode};

use super::session_convert::{from_persisted, to_persisted};
use crate::session_store::{PersistedPlaybackSession, PersistedSessionSnapshot, PersistedShellViewState};
use crate::shell::AppRuntime;

/// Capture the live queue + playback state and persist it via the active
/// session store. No-op when no session is active (`with_session_store` returns
/// `None`). Mirrors `crates/qbz/src/session_persist.rs::capture_and_save`, minus
/// the desktop-only `persist_session` gate (the daemon's store IS its queue
/// persistence, so it always saves).
pub async fn save_session_now<A: FrontendAdapter + Send + Sync + 'static>(
    runtime: &AppRuntime<A>,
) {
    let core = runtime.core();
    let (tracks, current_index) = core.get_all_queue_tracks().await;
    let full = core.get_queue_state_full().await;
    let ev = core.player().get_playback_event();
    let snapshot = PersistedSessionSnapshot {
        playback: PersistedPlaybackSession {
            queue_tracks: tracks.iter().map(to_persisted).collect(),
            current_index,
            current_position_secs: ev.position,
            volume: ev.volume,
            shuffle_enabled: full.shuffle,
            repeat_mode: repeat_to_str(full.repeat).to_string(),
            was_playing: ev.is_playing,
            saved_at: 0, // set inside save_session
        },
        // Shell-view columns are desktop-only; keep defaults so the schema
        // round-trips unchanged.
        shell_view: PersistedShellViewState::default(),
    };
    let saved = runtime.with_session_store(|s| s.save_session(&snapshot));
    match saved {
        Some(Ok(())) => {}
        Some(Err(e)) => log::warn!("[qbzd] driver: session save failed: {e}"),
        None => log::debug!("[qbzd] driver: session save skipped (no active session)"),
    }
}

/// Restore the persisted queue at boot, PAUSED (queue + order + repeat + volume,
/// never auto-playing). Returns `true` when a non-empty queue was restored.
/// Mirrors `session_persist::restore`'s Phase A; the daemon has no
/// `resume_playback_position` gate, so the saved position is threaded into the
/// snapshot but only replayed when the CLI later plays the restored track.
pub async fn restore_session_paused<A: FrontendAdapter + Send + Sync + 'static>(
    runtime: &AppRuntime<A>,
) -> bool {
    let Some(loaded) = runtime.with_session_store(|s| s.load_session()) else {
        return false; // no active session
    };
    let snapshot = match loaded {
        Ok(s) => s,
        Err(e) => {
            log::warn!("[qbzd] driver: session load failed: {e}");
            return false;
        }
    };
    let pb = snapshot.playback;
    if pb.queue_tracks.is_empty() {
        log::info!("[qbzd] driver: nothing to restore (saved queue is empty)");
        return false;
    }
    let count = pb.queue_tracks.len();
    let index = pb.current_index;
    let position = pb.current_position_secs;
    let tracks: Vec<QueueTrack> = pb.queue_tracks.into_iter().map(from_persisted).collect();
    let core = runtime.core();
    core.set_queue_with_order(tracks, index, pb.shuffle_enabled, None)
        .await;
    core.set_repeat_mode(repeat_from_str(&pb.repeat_mode)).await;
    let _ = core.set_volume(pb.volume);
    log::info!(
        "[qbzd] driver: restored {count} queue tracks (index {index:?}), paused; \
         saved position {position}s"
    );
    true
}

pub(super) fn repeat_to_str(mode: RepeatMode) -> &'static str {
    match mode {
        RepeatMode::Off => "off",
        RepeatMode::All => "all",
        RepeatMode::One => "one",
    }
}

fn repeat_from_str(s: &str) -> RepeatMode {
    match s {
        "all" => RepeatMode::All,
        "one" => RepeatMode::One,
        _ => RepeatMode::Off,
    }
}
