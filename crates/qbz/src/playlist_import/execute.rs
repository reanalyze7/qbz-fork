//! Step B: execute gate.

use slint::{ComponentHandle, Model};

use crate::{AppWindow, PlaylistImportState};

use super::session::{bump_generation, SESSION};

/// Everything the execute task needs, snapshotted on the event loop.
pub struct ExecuteArgs {
    pub url: String,
    pub name_override: Option<String>,
    /// Local folder id chosen in the dropdown ("" = no folder).
    pub folder_id: String,
    /// The run's generation (§1.8), carried by the sink and the
    /// completion arms.
    pub generation: u64,
}

/// Step B gate + reset (Svelte handleExecute's pre-invoke block).
/// Event-loop thread.
pub fn begin_execute(window: &AppWindow) -> Option<ExecuteArgs> {
    let state = window.global::<PlaylistImportState>();
    if state.get_loading() || state.get_import_completed() {
        return None;
    }
    let (url, name_override) = {
        let mut s = SESSION.lock().unwrap();
        let source_name = s.preview.as_ref()?.name.clone();
        // Rename goes out only when it differs from the source name; an
        // empty rename falls back to the source name (Appendix A).
        let custom = s.custom_name.trim().to_string();
        let name_override = if custom != source_name && !custom.is_empty() {
            Some(custom)
        } else {
            None
        };
        s.last_logged_percent = -1;
        (s.preview_url.clone(), name_override)
    };
    let folder_id = {
        let ids = state.get_folder_ids();
        ids.row_data(state.get_folder_index() as usize)
            .map(|s| s.to_string())
            .unwrap_or_default()
    };
    state.set_loading(true);
    state.set_error("".into());
    state.set_progress_visible(true);
    Some(ExecuteArgs {
        url,
        name_override,
        folder_id,
        generation: bump_generation(),
    })
}
