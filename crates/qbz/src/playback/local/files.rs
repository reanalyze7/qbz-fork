//! Audible step for a LOCAL user file, split out of the local-playback
//! cluster to keep sibling files under the line budget.

use super::super::loading::clear_loading;
use super::super::Runtime;
use crate::AppWindow;

/// Audible step for a LOCAL user file: read it off-thread and hand the bytes
/// to the player's `play_data` seam (which extracts the sample rate + drives
/// the PROTECTED device init, untouched here). CUE virtual tracks share one
/// file, so seek to the track start. `row_id` is the library row id. Called
/// by `play_audible` when the current queue track's source is `"local"`.
pub(in super::super) async fn play_local_file_audible(
    runtime: &Runtime,
    weak: &slint::Weak<AppWindow>,
    row_id: u64,
) {
    // Ephemeral tracks (synthetic id >= 2^48) resolve from the in-memory
    // session, never the DB. Everything downstream (read bytes, play_data, CUE
    // seek) is identical to a real local file.
    //
    // FAST PATH for CUE virtual tracks: all the tracks of a CUE album share ONE
    // big audio file. If that file is already loaded in the player (the loaded
    // track is ephemeral and points at the same path), DON'T re-read + re-decode
    // the whole FLAC — just seek to the new track's start. Re-reading a multi-
    // hundred-MB single-file album on every track click was "infierno de lento".
    // The seekbar then reports absolute file time (accepted limitation, as in
    // the Tauri build); the now-playing title/artist still update from the queue
    // cursor.
    if crate::ephemeral::is_ephemeral_id(row_id as i64) {
        if let Some(target) = crate::ephemeral::get_track(row_id as i64) {
            let loaded_id = runtime.core().player().state.current_track_id();
            if runtime.core().player().has_loaded_audio()
                && crate::ephemeral::is_ephemeral_id(loaded_id as i64)
                && crate::ephemeral::get_track(loaded_id as i64)
                    .map(|l| l.file_path == target.file_path)
                    .unwrap_or(false)
            {
                let pos = target.cue_start_secs.unwrap_or(0.0).max(0.0);
                let _ = runtime.core().player().seek(pos as u64);
                return;
            }
        }
    }
    let info = if crate::ephemeral::is_ephemeral_id(row_id as i64) {
        crate::ephemeral::get_track(row_id as i64).map(|t| (t.file_path, t.cue_start_secs))
    } else {
        tokio::task::spawn_blocking(move || {
            crate::library_db::with_db(|db| db.get_track(row_id as i64))
        })
        .await
        .ok()
        .flatten()
        .flatten()
        .map(|t| (t.file_path, t.cue_start_secs))
    };
    let Some((path, cue)) = info else {
        log::error!("[qbz-slint] local play: track {row_id} not found");
        clear_loading(weak, row_id);
        return;
    };
    // DSD (.dsf/.dff): split into its own helper (see `dsd.rs`) to keep this
    // file under the line budget.
    if qbz_dsd::is_dsd_path(std::path::Path::new(&path)) {
        super::dsd::play_dsd(runtime, weak, path, row_id).await;
        return;
    }
    // PLAYBACK LOCK (owner verdict 2026-06-10): the library never hides
    // network-folder content, so an unmounted drive surfaces HERE — one cheap
    // `Path::exists()` stat before the read, with friendly feedback instead
    // of a silent log-only failure. Runs inside spawn_blocking, never on the
    // audio callback thread: an unmounted path returns false instantly; only
    // a dead-but-still-mounted NFS/CIFS share could block, and then it blocks
    // a pool thread, not audio (see `local_track_file_exists`).
    let read_path = path.clone();
    let bytes = tokio::task::spawn_blocking(move || {
        if !std::path::Path::new(&read_path).exists() {
            return None;
        }
        std::fs::read(&read_path).ok()
    })
    .await
    .ok()
    .flatten();
    let Some(bytes) = bytes else {
        log::error!("[qbz-slint] local play: file not available at {path}");
        crate::toast::show_weak(
            weak,
            qbz_i18n::t("File not available — is the drive mounted?"),
            crate::ToastKind::Warning,
        );
        clear_loading(weak, row_id);
        return;
    };
    if let Err(e) = runtime.core().player().play_data(bytes, row_id) {
        log::error!("[qbz-slint] local play: play_data {row_id} failed: {e}");
        clear_loading(weak, row_id);
        return;
    }
    if let Some(start) = cue {
        if start > 0.0 {
            tokio::time::sleep(std::time::Duration::from_millis(120)).await;
            let _ = runtime.core().player().seek(start as u64);
        }
    }
}
