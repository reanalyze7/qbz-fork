//! Per-user Intelligent Search lifecycle + access wrapper (Phase 4 bridge).
//!
//! A process-global singleton over the headless
//! `qbz_app::settings::search_service::SearchService` (ADR-006: the cache
//! (Capa A) + ranking (Capa B) model logic lives in `qbz-app`; this module
//! only owns the per-user store lifecycle and the thin accessors the Slint
//! search surfaces — the cortinilla and the SWR result-page controller — call).
//!
//! Lifecycle mirrors `artist_blacklist` / `fav_cache` / `discover_prefs`: a
//! process-global `Mutex<Option<Service>>` bound per session via [`init`] /
//! [`teardown`], next to the other per-user stores. `SearchService` carries no
//! interior `Mutex` (the headless layer is deliberately plain); the `Mutex`
//! here is what gives the `&mut self` cache/ranking writes their exclusive
//! access. The `enabled` kill switch is an interior `AtomicBool`, so
//! [`set_enabled`] / [`is_enabled`] only need a shared `&self`.
//!
//! Fail-safe everywhere: with no session bound (`None`) every accessor behaves
//! as "disabled" — `cached`/`top_for_query` return `None`, `store`/`record`/
//! `rank_within` are no-ops, and [`is_enabled`] returns `false` so the
//! cortinilla never fires without a bound, enabled service.

mod accessors;
mod lifecycle;
#[cfg(test)]
mod tests;

// Re-export so qbz-slint imports the action enum from ONE place.
pub use qbz_app::settings::search_service::InteractionAction;

pub use accessors::{cached, is_enabled, rank_within, record, set_enabled, store, top_for_query};
pub use lifecycle::{init, teardown};
