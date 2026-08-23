//! Per-user "Not interested" dismissal store for Discover > Recommendations.
//!
//! Reco-SCOPED dismissal — deliberately NOT the app-wide blacklist: a
//! dismissed artist only leaves the two Recommended-Artist rails ("More like
//! the artists you love" / "Based on what you've been into lately"); it stays
//! visible in search, home, and label pages. The paint choke point in
//! `crate::external_reco` folds [`ids_snapshot`] into its exclusion set, and
//! the Blacklist Manager's "Recommendations" tab lists / undoes entries.
//!
//! Shape follows the light `discovery_dismiss` precedent (one small JSON file,
//! a full read on each op — no SQLite, no change-notify) but bound PER-USER
//! like `fav_cache` / `artist_blacklist`: a process-global path set via
//! [`init_for_user`] / dropped by [`teardown`]. Fail-open everywhere: with no
//! session bound (or a corrupt file) reads are empty and mutations no-op.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

mod io;
mod ops;

#[cfg(test)]
mod tests;

pub use ops::*;

/// JSON file name inside the per-user data dir.
const FILE_NAME: &str = "reco_dismiss.json";

/// One dismissed artist. `image_url` is optional (used only if a future
/// surface wants a thumbnail; the manager tab renders a generic avatar).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DismissedArtist {
    #[serde(default)]
    pub artist_id: u64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub image_url: String,
}

#[derive(Default, Serialize, Deserialize)]
struct DismissStore {
    #[serde(default)]
    artists: Vec<DismissedArtist>,
}

/// The bound per-user file. `None` outside an active session (pure fail-open
/// window), matching the `fav_cache` lifecycle.
static STORE_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);

/// Bind the per-user store path from `<dir>/reco_dismiss.json`. Called on
/// every session activation — login, restore, AND offline entry — next to
/// `fav_cache::init_for_user`.
pub fn init_for_user(base_dir: &Path) {
    if let Ok(mut guard) = STORE_PATH.lock() {
        *guard = Some(base_dir.join(FILE_NAME));
    }
}

/// Drop the binding on logout. Mirrors `fav_cache::teardown`.
pub fn teardown() {
    if let Ok(mut guard) = STORE_PATH.lock() {
        *guard = None;
    }
}
