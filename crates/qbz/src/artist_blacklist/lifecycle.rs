//! Per-session bind/unbind + the shared accessor/mutation helpers.

use std::path::Path;
use std::sync::Mutex;

use qbz_app::settings::artist_blacklist::{BlacklistService, DB_FILE_NAME};

/// Per-user blacklist service. `None` outside an active session (online or
/// offline); pure fail-open behavior in that window.
pub(super) static SERVICE: Mutex<Option<BlacklistService>> = Mutex::new(None);

/// The exact error string the Tauri build returns for a mutation attempted
/// with no active session. Kept verbatim so the UI surfaces the same message.
pub(super) const NO_SESSION_ERR: &str = "No active session - please log in";

/// Bind the per-user store from `<dir>/artist_blacklist.db`. Called on every
/// session activation — login, restore, AND offline entry — next to
/// `fav_cache::init_for_user`. Best-effort: a store-open failure logs and
/// leaves the singleton `None` (fail-open: nothing is blacklisted, the feature
/// reads as enabled, never blocks entry). The offline binding is the fix for
/// the Tauri gap where the blacklist was never initialized in offline mode.
pub fn init_for_user(base_dir: &Path) {
    let db_path = base_dir.join(DB_FILE_NAME);
    match BlacklistService::new(&db_path) {
        Ok(service) => {
            if let Ok(mut guard) = SERVICE.lock() {
                *guard = Some(service);
            }
        }
        Err(e) => log::error!("[qbz-slint] artist blacklist store open failed: {e}"),
    }
}

/// Drop the per-user store on logout. Mirrors `fav_cache::teardown`.
pub fn teardown() {
    if let Ok(mut guard) = SERVICE.lock() {
        *guard = None;
    }
}

/// Run a closure against the bound service, or `default` when there is none /
/// the lock is poisoned.
pub(super) fn with_service<T>(default: T, f: impl FnOnce(&BlacklistService) -> T) -> T {
    SERVICE
        .lock()
        .ok()
        .and_then(|guard| guard.as_ref().map(f))
        .unwrap_or(default)
}

/// Run a mutation against the bound service, returning the Tauri "no active
/// session" error string when there is none / the lock is poisoned.
pub(super) fn mutate(f: impl FnOnce(&BlacklistService) -> Result<(), String>) -> Result<(), String> {
    match SERVICE.lock() {
        Ok(guard) => match guard.as_ref() {
            Some(service) => f(service),
            None => Err(NO_SESSION_ERR.into()),
        },
        Err(_) => Err(NO_SESSION_ERR.into()),
    }
}
