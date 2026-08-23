//! File-level read/write for the dismiss store — fail-open on error.

use super::{DismissStore, STORE_PATH};
use std::path::PathBuf;

pub(super) fn store_path() -> Option<PathBuf> {
    STORE_PATH.lock().ok().and_then(|g| g.clone())
}

/// Fail-open read: no binding, unreadable file, or unknown/corrupt format all
/// yield an empty store (a corrupted file never blocks recommendations — the
/// user simply re-dismisses).
pub(super) fn load_store() -> DismissStore {
    let Some(path) = store_path() else {
        return DismissStore::default();
    };
    match std::fs::read(&path) {
        Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
        Err(_) => DismissStore::default(),
    }
}

pub(super) fn write_store(store: &DismissStore) {
    let Some(path) = store_path() else {
        return;
    };
    if let Some(parent) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            log::warn!("[qbz-slint] reco-dismiss dir failed: {e}");
            return;
        }
    }
    match serde_json::to_vec_pretty(store) {
        Ok(json) => {
            if let Err(e) = std::fs::write(&path, json) {
                log::warn!("[qbz-slint] reco-dismiss write failed: {e}");
            }
        }
        Err(e) => log::warn!("[qbz-slint] reco-dismiss serialize failed: {e}"),
    }
}
