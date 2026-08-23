//! Tree rail multi-select store (path -> selected `LocalTrack` record).

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

// Selected tracks in the tree (path -> record), for the bulk bar.
static TREE_SELECTED: LazyLock<Mutex<HashMap<String, qbz_library::LocalTrack>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub(crate) fn tree_selected() -> std::sync::MutexGuard<'static, HashMap<String, qbz_library::LocalTrack>> {
    TREE_SELECTED.lock().unwrap_or_else(|e| e.into_inner())
}

/// Snapshot the currently-selected tree tracks (scan order by path).
pub fn tree_selected_snapshot() -> Vec<qbz_library::LocalTrack> {
    let sel = tree_selected();
    let mut v: Vec<qbz_library::LocalTrack> = sel.values().cloned().collect();
    v.sort_by(|a, b| a.file_path.cmp(&b.file_path));
    v
}
