//! Discover per-tab section preferences (the "configurator" model).
//!
//! Frontend-agnostic port of the Tauri `discovery-v2/sectionPrefs.ts` store
//! (ADR-006): the ordered, per-tab list of `{ id, enabled }` that drives which
//! Discover rows show and in what order on each of the three tabs
//! (Home / Editor's Picks / For You). All tabs render from the SAME fetched
//! data — a tab is just a curated, ordered subset.
//!
//! Persistence is a single JSON blob in a per-user SQLite database
//! (`<base>/discover_prefs.db`), mirroring the other per-user settings stores
//! in this module. The blob shape is identical to the Tauri localStorage value
//! (`{ "home": [{id,enabled}], "editorPicks": [...], "forYou": [...] }`), so the
//! migration/reconcile logic ports verbatim and a profile could be shared.
//!
//! The model logic (defaults, migrate, reconcile, toggle, move, reset) is PURE
//! and headless-testable; the store is a thin wrapper.

mod defaults;
mod json;
mod model;
mod ops;
mod section_id;
mod store;
mod tabs;
#[cfg(test)]
mod tests;

pub use json::reconcile_list;
pub use model::{default_prefs, DiscoverPrefs, SectionPref};
pub use section_id::DiscoverySectionId;
pub use store::{create_empty_discover_prefs_state, DiscoverPrefsState, DiscoverPrefsStore};
pub use tabs::DiscoveryTab;
