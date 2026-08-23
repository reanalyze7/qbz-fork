//! Per-user store I/O: bind active user, read/write the collection-id-keyed map.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};

use super::model::Prefs;

/// The active user id, set by `init_for_user`. `None` before login (the store
/// degrades to defaults — there is no pre-login detail view).
static USER_ID: LazyLock<Mutex<Option<u64>>> = LazyLock::new(|| Mutex::new(None));

/// `<data_dir>/qbz/users/<user_id>/collection_view_prefs.json` for the active
/// user. `None` before login or when the data dir is unavailable.
fn store_path() -> Option<PathBuf> {
    let user_id = (*USER_ID.lock().ok()?)?;
    Some(
        dirs::data_dir()?
            .join("qbz")
            .join("users")
            .join(user_id.to_string())
            .join("collection_view_prefs.json"),
    )
}

/// Read the whole `{ collection-id -> Prefs }` map. A missing / unreadable /
/// unparseable file degrades to an empty map.
fn read_all() -> HashMap<String, Prefs> {
    let Some(path) = store_path() else {
        return HashMap::new();
    };
    match std::fs::read(&path) {
        Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
        Err(_) => HashMap::new(),
    }
}

/// Persist the whole map (best-effort — failures are logged).
fn write_all(map: &HashMap<String, Prefs>) {
    let Some(path) = store_path() else {
        log::warn!("[qbz-slint] collection view-prefs: no active user, not saving");
        return;
    };
    if let Some(parent) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            log::error!("[qbz-slint] collection view-prefs: create dir failed: {e}");
            return;
        }
    }
    match serde_json::to_vec_pretty(map) {
        Ok(json) => {
            if let Err(e) = std::fs::write(&path, json) {
                log::error!("[qbz-slint] collection view-prefs: write failed: {e}");
            }
        }
        Err(e) => log::error!("[qbz-slint] collection view-prefs: serialize failed: {e}"),
    }
}

/// Bind the store to `user_id` on shell entry. Subsequent reads/writes target
/// that user's JSON file.
pub fn init_for_user(user_id: u64) {
    if let Ok(mut guard) = USER_ID.lock() {
        *guard = Some(user_id);
    }
}

/// Load the stored prefs for `id`, or the §18 defaults when none are stored.
pub fn load(id: &str) -> Prefs {
    read_all().remove(id).unwrap_or_default()
}

/// Persist the prefs for `id` (read-modify-write the whole map). Writing the
/// default set is harmless (re-open restores the same defaults), so the caller
/// need not special-case it.
pub fn save(id: &str, prefs: &Prefs) {
    if id.is_empty() {
        return;
    }
    let mut map = read_all();
    map.insert(id.to_string(), prefs.clone());
    write_all(&map);
}

/// Remove the stored prefs for `id` (cleanup on collection delete, spec §18 /
/// §11.3). No-op when the key is absent.
pub fn remove(id: &str) {
    if id.is_empty() {
        return;
    }
    let mut map = read_all();
    if map.remove(id).is_some() {
        write_all(&map);
    }
}
