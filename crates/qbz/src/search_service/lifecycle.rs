//! Per-session bind/unbind of the search service singleton.

use std::path::Path;
use std::sync::Mutex;

use qbz_app::settings::search_service::SearchService;

/// Per-user search service. `None` outside an active session (online or
/// offline); every accessor reads as disabled in that window.
pub(super) static SERVICE: Mutex<Option<SearchService>> = Mutex::new(None);

// ---------------------------------------------------------------------------
// Lifecycle (mirrors artist_blacklist::{init_for_user, teardown})
// ---------------------------------------------------------------------------

/// Bind the per-user search service rooted at `base_dir` (the per-user data
/// dir; each store owns its own sub-file/sub-dir underneath). Called on every
/// session activation — login, restore, AND offline entry — next to
/// `artist_blacklist::init_for_user`. `SearchService::new` never fails
/// (missing/corrupt persisted state degrades to empty), so this is
/// infallible. Idempotent: replaces any previously bound service.
///
/// `enabled` seeds the kill switch from the persisted `ui_prefs.intelligent_search`
/// preference, so the cortinilla starts in the user's last-chosen state.
pub fn init(base_dir: &Path, enabled: bool) {
    let service = SearchService::new(base_dir);
    service.set_enabled(enabled);
    if let Ok(mut guard) = SERVICE.lock() {
        *guard = Some(service);
    }
}

/// Drop the per-user search service on logout. Mirrors
/// `artist_blacklist::teardown`.
pub fn teardown() {
    if let Ok(mut guard) = SERVICE.lock() {
        *guard = None;
    }
}

/// Run a closure against the bound service through a shared `&`, or `default`
/// when there is none / the lock is poisoned.
pub(super) fn with_service<T>(default: T, f: impl FnOnce(&SearchService) -> T) -> T {
    SERVICE
        .lock()
        .ok()
        .and_then(|guard| guard.as_ref().map(f))
        .unwrap_or(default)
}

/// Run a closure against the bound service through a `&mut` (cache `put` /
/// ranking `record` need it; the `Mutex` lock provides the exclusivity). No-op
/// when there is none / the lock is poisoned.
pub(super) fn with_service_mut(f: impl FnOnce(&mut SearchService)) {
    if let Ok(mut guard) = SERVICE.lock() {
        if let Some(service) = guard.as_mut() {
            f(service);
        }
    }
}
