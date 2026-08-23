//! Accessors (fail-open when no session is bound).

use std::collections::HashSet;

use qbz_app::settings::pinned_items::PinnedItemsService;

use super::{PinnedItem, SERVICE};

/// Run a closure against the bound service, or `default` when there is none /
/// the lock is poisoned.
fn with_service<T>(default: T, f: impl FnOnce(&PinnedItemsService) -> T) -> T {
    SERVICE
        .lock()
        .ok()
        .and_then(|guard| guard.as_ref().map(f))
        .unwrap_or(default)
}

/// True when the `(kind, id)` item is pinned. Fail-open `false` when no
/// session is bound.
pub fn is_pinned(kind: &str, id: &str) -> bool {
    with_service(false, |s| s.is_pinned(kind, id))
}

/// All pinned items, newest first, for the Pinned section loader. Empty on no
/// session or query error.
pub fn list() -> Vec<PinnedItem> {
    with_service(Vec::new(), |s| s.list().unwrap_or_default())
}

/// Count of pinned items. `0` when no session is bound.
#[allow(dead_code)] // family-API parity (blacklist::count); no consumer wired yet
pub fn count() -> usize {
    with_service(0, |s| s.count())
}

/// Snapshot of the full `(kind, id)` key set, for bulk card stamping (mirrors
/// `artist_blacklist::ids_snapshot`). Empty when no session is bound.
#[allow(dead_code)] // converters stamp per-row via is_pinned today; kept for bulk maps
pub fn keys_snapshot() -> HashSet<(String, String)> {
    with_service(HashSet::new(), |s| s.keys_snapshot())
}
