//! Pure IO: JSON persistence for the per-context genre selection.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

#[derive(Default, Serialize, Deserialize)]
pub(super) struct Persisted {
    /// Per-context selections ("discover" / "favorites" / ...).
    #[serde(default)]
    pub(super) contexts: HashMap<String, Vec<u64>>,
    /// Legacy single-list selection — migrated into the "discover" context.
    #[serde(default)]
    pub(super) selected: Vec<u64>,
    #[serde(default = "default_true")]
    pub(super) remember: bool,
}

fn store_path() -> Option<PathBuf> {
    Some(dirs::data_dir()?.join("qbz").join("genre_filter.json"))
}

pub(super) fn load_persisted() -> Persisted {
    let Some(path) = store_path() else {
        return Persisted::default();
    };
    match std::fs::read(&path) {
        Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
        Err(_) => Persisted::default(),
    }
}

pub(super) fn save_persisted(contexts: &HashMap<String, Vec<u64>>, remember: bool) {
    let Some(path) = store_path() else {
        return;
    };
    if !remember {
        // Remember off — drop any persisted selection.
        let _ = std::fs::remove_file(&path);
        return;
    }
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let data = Persisted {
        contexts: contexts.clone(),
        selected: Vec::new(),
        remember,
    };
    if let Ok(json) = serde_json::to_vec_pretty(&data) {
        let _ = std::fs::write(&path, json);
    }
}
