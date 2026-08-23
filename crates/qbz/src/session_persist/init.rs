//! Store open + gate seeding at session activation, and the synchronous
//! exit-time flush.

use std::path::Path;
use std::sync::atomic::Ordering;

use qbz_app::session_store::SessionStore;
use qbz_app::settings::playback::PlaybackPreferencesStore;

use super::save::capture_and_save;
use super::state::{persist_enabled, set_gates, EXIT_CTX, PERSIST_SESSION, RESUME_POSITION, STORE};

/// Open the per-user session store and seed the gate flags synchronously from
/// the playback preferences. Called at session activation alongside the other
/// `init_for_user` stores. Failures degrade to "no persistence" (logged).
pub fn init_for_user(base_dir: &Path) {
    let opened = match SessionStore::new_at(base_dir) {
        Ok(store) => {
            *STORE.lock().unwrap() = Some(store);
            true
        }
        Err(e) => {
            log::warn!("[qbz-slint] session_persist: open failed: {e}");
            *STORE.lock().unwrap() = None;
            false
        }
    };
    // Seed the gates from the per-user playback prefs so capture/restore work
    // before the async settings snapshot has had a chance to call set_gates.
    match PlaybackPreferencesStore::new_at(base_dir).and_then(|s| s.get_preferences()) {
        Ok(prefs) => set_gates(prefs.persist_session, prefs.resume_playback_position),
        Err(e) => {
            log::warn!("[qbz-slint] session_persist: prefs read failed, gates off: {e}");
            set_gates(false, false);
        }
    }
    log::info!(
        "[qbz-slint] session_persist: init at {} (store_open={opened}, persist={}, resume={})",
        base_dir.display(),
        PERSIST_SESSION.load(Ordering::Relaxed),
        RESUME_POSITION.load(Ordering::Relaxed)
    );
}

/// Flush a final full snapshot synchronously (the window close handlers run on
/// the UI thread, off the tokio runtime, so we `block_on`). No-op until the exit
/// context is bound or unless `persist_session` is on.
pub fn save_on_exit() {
    if !persist_enabled() {
        return;
    }
    if let Some((runtime, handle)) = EXIT_CTX.get() {
        handle.block_on(capture_and_save(runtime));
    }
}
