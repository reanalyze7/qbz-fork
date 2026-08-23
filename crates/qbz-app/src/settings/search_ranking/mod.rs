//! Per-query interaction ranking for the Intelligent Search module (Capa B).
//!
//! This is the frontend-agnostic, headless ranking layer (ADR-006). It learns,
//! per *normalized query*, which entities the user actually interacts with
//! (opens, plays, favorites) and uses that signal to reorder the **cortinilla**
//! (the inline suggestion strip) — NEVER the results page itself.
//!
//! ## Privacy
//!
//! Everything here is local: a single JSON file under the per-user data dir.
//! There is zero telemetry — no network, no analytics, no remote reporting.
//!
//! ## Persistence
//!
//! State is a `HashMap<normalized_query, HashMap<(kind, id), score>>` serialized
//! to `<base_dir>/search/search_ranking.json`. A missing or corrupt file loads
//! as empty state and never panics (same graceful-degradation discipline as
//! `discover_prefs` / `reco_store`). Writes are best-effort: a failure is logged
//! via `log::warn!` and swallowed, never propagated to the caller.
//!
//! ## Bounds (so the file can't grow unbounded)
//!
//! - Each `(kind, id)` score is capped at `MAX_SCORE` (1000).
//! - The number of distinct queries is LRU-bound to `MAX_QUERIES` (200); the
//!   least-recently-touched query is evicted when the cap is exceeded.
//!
//! ## Architecture note
//!
//! This struct owns ONLY Capa A's sibling (Capa B). It does NOT hold `QbzCore`
//! and does NOT call `search_all`. The SWR orchestration (render cached -> fire
//! live -> replace) lives in the qbz-slint controller. See the module-level
//! decision in the search module.

mod action;
mod ops;
mod schema;
mod store;
#[cfg(test)]
mod tests;
mod tunables;

pub use action::InteractionAction;
pub use store::SearchRanking;
pub use tunables::{MAX_QUERIES, MAX_SCORE, WEIGHT_FAVORITE, WEIGHT_OPEN, WEIGHT_PLAY};
