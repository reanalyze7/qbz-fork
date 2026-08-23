//! Lifecycle (mirrors artist_blacklist::{init_for_user, teardown}).

use std::path::Path;

use qbz_app::settings::pinned_items::{PinnedItemsService, DB_FILE_NAME};

use super::SERVICE;

/// Bind the per-user store from `<dir>/pinned_items.db`. Called on every
/// session activation — login, restore, AND offline entry — next to
/// `artist_blacklist::init_for_user`. Best-effort: a store-open failure logs
/// and leaves the singleton `None` (fail-open: nothing is pinned, never
/// blocks entry). The offline binding matters — pinned items are local-only
/// and must render offline.
pub fn init_for_user(base_dir: &Path) {
    let db_path = base_dir.join(DB_FILE_NAME);
    match PinnedItemsService::new(&db_path) {
        Ok(service) => {
            if let Ok(mut guard) = SERVICE.lock() {
                *guard = Some(service);
            }
        }
        Err(e) => log::error!("[qbz-slint] pinned items store open failed: {e}"),
    }
}

/// Drop the per-user store on logout. Mirrors `artist_blacklist::teardown`.
pub fn teardown() {
    if let Ok(mut guard) = SERVICE.lock() {
        *guard = None;
    }
}
