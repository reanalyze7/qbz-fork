//! Thin accessors over the bound search service (read-as-disabled when
//! no session is bound).

use super::lifecycle::{with_service, with_service_mut};
use super::InteractionAction;

/// Flip the master kill switch on the bound service. No-op when unbound — the
/// next [`super::init`] re-seeds the flag from the persisted preference
/// anyway. Works through a shared `&self` (interior `AtomicBool`), so it
/// does not need the exclusive `with_service_mut` path.
pub fn set_enabled(on: bool) {
    with_service((), |s| s.set_enabled(on));
}

/// True only when a service is bound AND it is enabled. The cortinilla gates
/// on this (fail-safe `false` when no session is bound).
pub fn is_enabled() -> bool {
    with_service(false, |s| s.enabled())
}

/// Cached merged result for `query`, or `None` when unbound / disabled /
/// uncached.
// KEPT, and it points at a real defect. `store()` IS called (search/load/
// cortinilla.rs), so the search cache is written on every query and never read
// back — this reader is the missing half. Removing it would make the
// write-only cache permanent and hide the bug; the fix is to call this from the
// cortinilla load path before hitting the network.
#[allow(dead_code)]
pub fn cached(query: &str) -> Option<qbz_models::SearchAllResults> {
    with_service(None, |s| s.cached(query))
}

/// Store a live `results` page for `query` in the cache. No-op when unbound /
/// disabled.
pub fn store(query: &str, results: &qbz_models::SearchAllResults) {
    with_service_mut(|s| s.store(query, results));
}

/// Record a user interaction with a search-surfaced entity. No-op when
/// unbound / disabled. `kind` is one of `"artist" | "album" | "track" | "playlist"`.
pub fn record(query: &str, kind: &str, id: &str, action: InteractionAction) {
    with_service_mut(|s| s.record_interaction(query, kind, id, action));
}

/// The single highest-scored `(kind, id)` learned for `query`, or `None` when
/// unbound / disabled / nothing learned.
pub fn top_for_query(query: &str) -> Option<(String, String)> {
    with_service(None, |s| s.top_for_query(query))
}

/// Stable-sort `items` in place by their learned score for `query` (the
/// cortinilla reorder). No-op when unbound / disabled.
pub fn rank_within<T>(query: &str, kind: &str, items: &mut Vec<T>, id_of: impl Fn(&T) -> String) {
    with_service((), |s| s.rank_within(query, kind, items, id_of));
}
