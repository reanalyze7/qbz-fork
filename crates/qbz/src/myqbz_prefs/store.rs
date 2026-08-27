//! Pure-JSON-file persistence layer for the "My Qoqobuz" branding.

use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};

use serde::{Deserialize, Serialize};

use super::DEFAULT_LABEL;

/// The active user id, set by `init_for_user`. `None` before login (the
/// store degrades to defaults — there is no pre-login branding surface).
static USER_ID: LazyLock<Mutex<Option<u64>>> = LazyLock::new(|| Mutex::new(None));

/// Persisted branding. Missing fields default sanely so an older file still
/// deserializes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct Branding {
    #[serde(default = "default_label")]
    pub(super) label: String,
    /// Absolute path to a custom icon, or empty for the default glyph.
    #[serde(default)]
    pub(super) icon_path: String,
}

fn default_label() -> String {
    DEFAULT_LABEL.to_string()
}

impl Default for Branding {
    fn default() -> Self {
        Self {
            label: default_label(),
            icon_path: String::new(),
        }
    }
}

/// `<data_dir>/qbz/users/<user_id>/myqbz_branding.json` for the active user.
/// `None` before login or when the data dir is unavailable.
fn store_path() -> Option<PathBuf> {
    let user_id = (*USER_ID.lock().ok()?)?;
    Some(
        dirs::data_dir()?
            .join("qbz")
            .join("users")
            .join(user_id.to_string())
            .join("myqbz_branding.json"),
    )
}

/// Load the active user's branding. A missing / unreadable / unparseable file
/// degrades to defaults.
pub(super) fn read() -> Branding {
    let Some(path) = store_path() else {
        return Branding::default();
    };
    match std::fs::read(&path) {
        Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
        Err(_) => Branding::default(),
    }
}

/// Persist the branding (best-effort — failures are logged).
pub(super) fn write(b: &Branding) {
    let Some(path) = store_path() else {
        log::warn!("[qbz-slint] myqbz branding: no active user, not saving");
        return;
    };
    if let Some(parent) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            log::error!("[qbz-slint] myqbz branding: create dir failed: {e}");
            return;
        }
    }
    match serde_json::to_vec_pretty(b) {
        Ok(json) => {
            if let Err(e) = std::fs::write(&path, json) {
                log::error!("[qbz-slint] myqbz branding: write failed: {e}");
            }
        }
        Err(e) => log::error!("[qbz-slint] myqbz branding: serialize failed: {e}"),
    }
}

/// Bind the store to `user_id` on shell entry. Subsequent reads/writes target
/// that user's JSON file.
pub fn init_for_user(user_id: u64) {
    if let Ok(mut guard) = USER_ID.lock() {
        *guard = Some(user_id);
    }
}
