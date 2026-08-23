//! Per-session store bind/unbind.

use std::path::Path;

use qbz_app::settings::reco_store::RecoStore;

use super::RECO;

/// Open the per-user reco store (`<base_dir>/reco/events.db`). Best-effort: a
/// failure logs and leaves reco disabled (every helper then degrades to
/// no-op). Called on every session activation — login, restore, offline
/// entry — next to `fav_cache::init_for_user`.
pub fn init_for_user(base_dir: &Path) {
    match RecoStore::new_at(base_dir) {
        Ok(store) => {
            log::info!("[reco] event store opened for session");
            if let Ok(mut guard) = RECO.lock() {
                *guard = Some(store);
            }
        }
        Err(e) => log::warn!("[reco] init failed, reco disabled: {e}"),
    }
}

/// Drop the per-user store on logout (mirrors `fav_cache::teardown`).
pub fn teardown() {
    if let Ok(mut guard) = RECO.lock() {
        *guard = None;
    }
}
