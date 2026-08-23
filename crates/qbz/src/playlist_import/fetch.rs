//! Step A: preview fetch gate + result handlers.

use slint::{ComponentHandle, ModelRc, VecModel};

use qbz_playlist_import::{detect_provider_key, ImportPlaylist};

use crate::{AppWindow, ImportLogEntry, PlaylistImportState};

use super::format::{clear_summary, provider_display_name, push_log};
use super::session::SESSION;

/// Step A gate + reset (Svelte handlePreview's pre-invoke block). Returns
/// the URL to fetch, or None when the gate fails. Event-loop thread.
pub fn begin_fetch(window: &AppWindow) -> Option<String> {
    let state = window.global::<PlaylistImportState>();
    if state.get_loading() || !state.get_can_fetch() {
        return None;
    }
    let url = state.get_url().to_string();
    let detected = detect_provider_key(&url)?;
    {
        let mut s = SESSION.lock().unwrap();
        s.preview = None;
        s.preview_url.clear();
        s.locked_provider = Some(detected);
    }
    state.set_loading(true);
    state.set_error("".into());
    state.set_show_preview(false);
    state.set_import_completed(false);
    state.set_active_provider(detected.as_str().into());
    state.set_has_progress(false);
    state.set_progress(0.0);
    state.set_status_line("".into());
    state.set_current_track("".into());
    clear_summary(window);
    state.set_log(ModelRc::new(VecModel::from(Vec::<ImportLogEntry>::new())));
    push_log(window, qbz_i18n::t("Checking playlist link..."), "info");
    state.set_progress_visible(true);
    Some(url)
}

/// Preview fetch succeeded (Svelte handlePreview's try arm). Event-loop.
pub fn apply_preview_ok(window: &AppWindow, url: &str, preview: ImportPlaylist) {
    let state = window.global::<PlaylistImportState>();
    let count = preview.tracks.len();
    let provider = provider_display_name(&preview.provider);
    state.set_custom_name(preview.name.as_str().into());
    {
        let mut s = SESSION.lock().unwrap();
        s.custom_name = preview.name.clone();
        s.preview_url = url.trim().to_string();
        s.preview = Some(preview);
    }
    push_log(
        window,
        qbz_i18n::t_args("Found {} tracks from {}.", &[&count.to_string(), provider]),
        "success",
    );
    state.set_loading(false);
    // The URL input is disabled during the fetch, so it still equals the
    // fetched URL — step B (rename + Import) becomes visible.
    state.set_show_preview(
        state.get_url().trim() == SESSION.lock().unwrap().preview_url.as_str(),
    );
}

/// Preview fetch failed (Svelte handlePreview's catch arm). Event-loop.
pub fn apply_preview_err(window: &AppWindow, err: &str) {
    let state = window.global::<PlaylistImportState>();
    state.set_error(err.into());
    push_log(window, qbz_i18n::t_args("Import failed: {}", &[err]), "error");
    state.set_loading(false);
}
