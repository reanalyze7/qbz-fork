//! Filesystem I/O for the JSON-backed UI preference store.

use std::path::PathBuf;

use super::model::UiPrefs;

/// Resolve `<data_dir>/qbz/ui_prefs.json`.
fn prefs_path() -> Option<PathBuf> {
    Some(dirs::data_dir()?.join("qbz").join("ui_prefs.json"))
}

/// Load the UI preferences. A missing or unreadable file degrades to
/// `UiPrefs::default()` rather than erroring.
pub fn load() -> UiPrefs {
    let Some(path) = prefs_path() else {
        return UiPrefs::default();
    };
    match std::fs::read_to_string(&path) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_else(|e| {
            log::warn!("[qbz-slint] ui_prefs.json parse failed, using defaults: {e}");
            UiPrefs::default()
        }),
        Err(_) => UiPrefs::default(),
    }
}

/// Persist the UI preferences. Best-effort — failures are logged.
pub fn save(prefs: &UiPrefs) {
    let Some(path) = prefs_path() else {
        log::warn!("[qbz-slint] ui_prefs.json: data dir unavailable, not saving");
        return;
    };
    if let Some(parent) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            log::error!("[qbz-slint] ui_prefs.json: create dir failed: {e}");
            return;
        }
    }
    match serde_json::to_string_pretty(prefs) {
        Ok(text) => {
            if let Err(e) = std::fs::write(&path, text) {
                log::error!("[qbz-slint] ui_prefs.json: write failed: {e}");
            }
        }
        Err(e) => log::error!("[qbz-slint] ui_prefs.json: serialize failed: {e}"),
    }
}
