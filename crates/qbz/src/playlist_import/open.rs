//! `open` (modal reset) + the URL/name edit handlers.

use slint::{ComponentHandle, Model, ModelRc, VecModel};

use qbz_playlist_import::detect_provider_key;

use crate::{AppWindow, ImportLogEntry, PlaylistImportState, SidebarState};

use super::format::clear_summary;
use super::session::{bump_generation, Session, SESSION};

/// Open the modal fully reset — Tauri remounts the Svelte component on
/// every open, so nothing persists. Event-loop thread.
pub fn open(window: &AppWindow) {
    // Invalidate any in-flight run's modal writes before resetting.
    bump_generation();
    *SESSION.lock().unwrap() = Session::default();

    let state = window.global::<PlaylistImportState>();
    state.set_url("".into());
    state.set_custom_name("".into());
    state.set_loading(false);
    state.set_error("".into());
    state.set_active_provider("".into());
    state.set_can_fetch(false);
    state.set_show_preview(false);
    state.set_import_completed(false);
    state.set_progress_visible(false);
    state.set_has_progress(false);
    state.set_progress(0.0);
    state.set_status_line("".into());
    state.set_current_track("".into());
    state.set_log(ModelRc::new(VecModel::from(Vec::<ImportLogEntry>::new())));
    clear_summary(window);

    // Folder dropdown from the sidebar's folder list — the exact
    // create-playlist builder pattern: index 0 = "No folder" (id "").
    let folders = window.global::<SidebarState>().get_folders();
    let mut opts: Vec<slint::SharedString> = vec![qbz_i18n::t("No folder").into()];
    let mut ids: Vec<slint::SharedString> = vec!["".into()];
    for i in 0..folders.row_count() {
        if let Some(f) = folders.row_data(i) {
            opts.push(f.name);
            ids.push(f.id);
        }
    }
    state.set_folder_options(ModelRc::new(VecModel::from(opts)));
    state.set_folder_ids(ModelRc::new(VecModel::from(ids)));
    state.set_folder_index(0);

    state.set_open(true);
}

/// Recompute the URL-derived properties on every keystroke (Svelte's
/// derived detectedProvider / activeProvider / isValid / showPreview),
/// plus the post-completion fresh-import reset path. Event-loop thread.
pub fn on_url_edited(window: &AppWindow, text: &str) {
    let state = window.global::<PlaylistImportState>();
    let trimmed = text.trim();
    let detected = detect_provider_key(text);

    let mut s = SESSION.lock().unwrap();

    // After a completed import, editing the URL away from the imported
    // one rearms the modal for a fresh import without reopening.
    if state.get_import_completed() && trimmed != s.last_imported_url {
        s.locked_provider = None;
        state.set_import_completed(false);
        state.set_error("".into());
        state.set_log(ModelRc::new(VecModel::from(Vec::<ImportLogEntry>::new())));
        state.set_progress_visible(false);
        state.set_has_progress(false);
        state.set_progress(0.0);
        state.set_status_line("".into());
        state.set_current_track("".into());
        clear_summary(window);
    }

    let active = s.locked_provider.or(detected);
    state.set_active_provider(active.map(|p| p.as_str()).unwrap_or("").into());
    state.set_can_fetch(detected.is_some() && !crate::offline_mode::engine().is_offline());
    state.set_show_preview(s.preview.is_some() && trimmed == s.preview_url);
}

/// Keep the session's rename mirror fresh (read back by
/// [`super::execute::begin_execute`]). Fired on every name-LineEdit
/// keystroke.
pub fn on_name_edited(text: &str) {
    SESSION.lock().unwrap().custom_name = text.to_string();
}
