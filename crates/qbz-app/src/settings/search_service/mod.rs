//! SearchService — the composed, frontend-agnostic Intelligent Search facade.
//!
//! This is the single reusable entry point that owns the two headless layers of
//! Intelligent Search (ADR-006):
//!
//! - **Capa A** — [`super::search_cache::SearchCache`]: stale-while-revalidate result cache.
//! - **Capa B** — [`super::search_ranking::SearchRanking`]: per-query interaction ranking for the
//!   cortinilla.
//!
//! ## What it deliberately does NOT own
//!
//! `SearchService` is **non-generic**. It does NOT hold a `QbzCore` and does NOT
//! call `core.search_all`. `QbzCore` is `QbzCore<A: FrontendAdapter>`; making
//! `SearchService<A>` would force that generic through every qbz-slint global
//! accessor for no benefit. The SWR orchestration (render cached → fire live →
//! replace, guarded by the version counter) lives in the qbz-slint controller,
//! which already calls `core.search_all()` itself. This struct is purely the
//! reusable cache + ranking layer.
//!
//! ## Interior mutability
//!
//! Cache `put` and ranking `record` need `&mut self`; the corresponding service
//! methods therefore take `&mut self`. There is intentionally NO interior
//! `Mutex` inside `SearchService` — the qbz-slint global wraps the whole service
//! in a `Mutex` (Phase 4), so the caller owns the locking. The only interior
//! mutability here is the [`std::sync::atomic::AtomicBool`] enabled flag, which
//! `set_enabled` / `enabled` can flip through a shared `&self` (the kill switch
//! must work even while another thread holds nothing but `&self`).

mod service;
#[cfg(test)]
mod tests;

// Re-export so qbz-slint imports the action enum from ONE place.
pub use super::search_ranking::InteractionAction;
pub use service::SearchService;
