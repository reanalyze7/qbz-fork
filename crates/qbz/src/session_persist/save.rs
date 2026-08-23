//! Full snapshot capture + the cheap position-only save.

use qbz_app::session_store::{PersistedPlaybackSession, PersistedSessionSnapshot, PersistedShellViewState};

use super::convert::{repeat_to_str, to_persisted};
use super::state::{persist_enabled, Runtime, STORE};

/// Capture the live queue + playback state and persist it. No-op unless
/// `persist_session` is on and the store is open. Async (reads the queue lock).
pub async fn capture_and_save(runtime: &Runtime) {
    if !persist_enabled() {
        return;
    }
    let (tracks, current_index) = runtime.core().get_all_queue_tracks().await;
    // Crash-chain level >=3 bypassed the restore this boot, so the queue on
    // disk is the GOOD copy the user wants back on a healthy start — don't
    // clobber it with this session's empty queue at exit. A queue the user
    // actually built during the recovered boot still saves normally.
    if tracks.is_empty() && crate::crash_chain_level() >= 3 {
        log::info!(
            "[qbz-slint] session_persist: crash-chain recovery boot with empty queue — \
             keeping the preserved snapshot on disk"
        );
        return;
    }
    let full = runtime.core().get_queue_state_full().await;
    let pb = runtime.core().get_playback_state();
    let snapshot = PersistedSessionSnapshot {
        playback: PersistedPlaybackSession {
            queue_tracks: tracks.iter().map(to_persisted).collect(),
            current_index,
            current_position_secs: pb.position,
            volume: pb.volume,
            shuffle_enabled: full.shuffle,
            repeat_mode: repeat_to_str(full.repeat).to_string(),
            was_playing: pb.is_playing,
            saved_at: 0, // set inside save_session
        },
        // Shell-view restoration is handled separately (ui_prefs startup_page);
        // keep the Tauri view columns at their defaults so the schema round-trips.
        shell_view: PersistedShellViewState::default(),
    };
    let track_count = snapshot.playback.queue_tracks.len();
    if let Some(store) = STORE.lock().unwrap().as_ref() {
        match store.save_session(&snapshot) {
            Ok(()) => log::info!(
                "[qbz-slint] session_persist: saved {track_count} queue tracks (pos {}s, playing {})",
                snapshot.playback.current_position_secs,
                snapshot.playback.was_playing
            ),
            Err(e) => log::warn!("[qbz-slint] session_persist: save failed: {e}"),
        }
    } else {
        log::warn!("[qbz-slint] session_persist: capture skipped (store not open)");
    }
}

/// Quick position-only save (a single cheap UPDATE) — for the poll loop's
/// throttled tick and the pause edge, so a crash keeps a near-current position.
pub fn save_position(position_secs: u64) {
    if !persist_enabled() {
        return;
    }
    if let Some(store) = STORE.lock().unwrap().as_ref() {
        let _ = store.save_position(position_secs);
    }
}
