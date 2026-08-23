//! Import completion handlers (success + failure).

use slint::ComponentHandle;

use qbz_playlist_import::ImportSummary;

use crate::{AppWindow, PlaylistImportState};

use super::format::{parts_line, push_log};
use super::session::SESSION;

/// Import finished (Svelte handleExecute's success arm): completion logs
/// + summary block + import-completed. Toast / sidebar refresh /
/// navigation live in the main.rs arm (§1.8: those fire even after
/// close; this fn is generation-guarded by the caller). Event-loop.
pub fn apply_execute_ok(window: &AppWindow, summary: &ImportSummary) {
    let state = window.global::<PlaylistImportState>();
    {
        let mut s = SESSION.lock().unwrap();
        s.last_imported_url = s.preview_url.clone();
    }
    state.set_import_completed(true);
    push_log(
        window,
        qbz_i18n::t_args(
            "Imported {} of {} tracks into QBZ.",
            &[
                &summary.matched_tracks.to_string(),
                &summary.total_tracks.to_string(),
            ],
        ),
        "success",
    );
    if !summary.qobuz_playlist_ids.is_empty() {
        if summary.parts_created > 1 {
            push_log(window, parts_line(summary.parts_created), "success");
        } else {
            push_log(window, qbz_i18n::t("Playlist created in Qobuz™."), "success");
        }
    } else {
        push_log(window, qbz_i18n::t("No matching tracks found."), "error");
    }
    // Summary block (pre-formatted; "" = hidden). `playlist_name` is the
    // name the playlist was created under — rename included (deliberate
    // owner fix vs the Tauri original, see qbz_playlist_import::importer).
    state.set_summary_playlist(qbz_i18n::t_args("Playlist: {}", &[&summary.playlist_name]).into());
    state.set_summary_matched(
        qbz_i18n::t_args(
            "Tracks matched: {} / {}",
            &[
                &summary.matched_tracks.to_string(),
                &summary.total_tracks.to_string(),
            ],
        )
        .into(),
    );
    state.set_summary_skipped(qbz_i18n::t_args("Skipped: {}", &[&summary.skipped_tracks.to_string()]).into());
    state.set_summary_parts(if summary.parts_created > 1 {
        parts_line(summary.parts_created).into()
    } else {
        "".into()
    });
    // The bar/status hide with loading, as in Tauri (`loading` gates the
    // bar there).
    state.set_has_progress(false);
    state.set_status_line("".into());
    state.set_current_track("".into());
    state.set_loading(false);
}

/// Import failed (Svelte handleExecute's catch arm). The error toast
/// lives in the main.rs arm. Event-loop thread.
pub fn apply_execute_err(window: &AppWindow, err: &str) {
    let state = window.global::<PlaylistImportState>();
    state.set_error(err.into());
    push_log(window, qbz_i18n::t_args("Import failed: {}", &[err]), "error");
    state.set_has_progress(false);
    state.set_status_line("".into());
    state.set_current_track("".into());
    state.set_loading(false);
}
