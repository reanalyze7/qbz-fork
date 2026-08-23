//! Per-collection view-prefs persistence for the My QBZ DETAIL view
//! (spec 12 §18).
//!
//! Mirrors the Tauri `userStorage` key `collection-view-prefs.{collectionId}`:
//! each collection remembers its own toolbar state across opens. The persisted
//! shape (five fields) is exactly the Tauri set:
//!
//!   { viewMode, sortBy, sortDir, typeFilter, sourceFilter:[SourceKind] }
//!
//! `searchQuery` and `selectMode` are intentionally TRANSIENT — never persisted
//! (same as Tauri).
//!
//! Storage is per-user JSON (so different Qobuz accounts keep independent
//! prefs), scoped the same way as `myqbz_prefs.rs`:
//!
//!   <data_dir>/qbz/users/<user_id>/collection_view_prefs.json
//!
//! Rather than one file per collection, the whole map lives in one tiny JSON
//! (`{ "<collection-id>": { … } }`) — read-modify-write on every set. The store
//! is keyed by collection id, which is the §18 contract.
//!
//! Lifecycle (driven from `myqbz_detail` + `myqbz_edit`):
//!  - **restore on open**: `load(id)` → apply each field, else defaults.
//!  - **persist on change**: `save(id, prefs)` after a toolbar setter mutates a
//!    persisted field — gated behind a `hydrated` flag so the restore is not
//!    clobbered by an early persist (mirrors Tauri's `prefsHydrated`).
//!  - **clear on delete**: `remove(id)` drops the orphaned key.

mod model;
mod store;

#[cfg(test)]
mod tests;

pub use model::Prefs;
pub use store::{init_for_user, load, remove, save};
