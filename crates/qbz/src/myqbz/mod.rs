//! My QBZ controller — the Mixtapes & Collections index grids (read-only in
//! this slice). Mirrors `crate::playlist_manager`: it loads `MixtapeCollection`
//! rows from the per-user `library.db` via `qbz_mixtape::repo::list_collections`
//! (called through `crate::library_db::with_db` + `with_connection`), precomputes
//! every display string (eyebrow label, "N albums" ICU plural, pre-downscaled
//! mosaic cover URLs) and pushes ready-to-render `MixtapeCardItem`s into
//! `MyQbzState`. The views do NO per-row lookups.
//!
//! READ-ONLY SCOPE (Phase-2 Slice 2): create-new + open-detail are wired as
//! logging STUBS (`open_card` / `create_*`). The sidebar nav routes here and
//! loads real data; that is the testable path for this slice.
//!
//! The backend (`qbz-mixtape`) is reused directly — no Tauri command wrappers
//! (ADR-005), headless (ADR-006). The repo hydrates each collection's items so
//! counts + mosaic artwork are accurate (`repo.rs` `list_collections`).

mod artwork_jobs;
mod card;
mod db;
mod labels;
mod navigate;
mod offline;
mod render;
mod sort_filter;

/// Which grid a navigation targets.
#[derive(Clone, Copy, PartialEq)]
pub enum Grid {
    Mixtapes,
    Collections,
}

pub use artwork_jobs::artwork_jobs;
pub use card::set_mosaic_cover;
pub use db::{create_collection, kind_from_str, list_collections};
pub use labels::small_qobuz_url;
pub use navigate::navigate;
pub use offline::{offline_availability, retain_available_offline, OfflineAvailability};
pub use render::{apply, rebuild, reset, set_loading, set_sort};
