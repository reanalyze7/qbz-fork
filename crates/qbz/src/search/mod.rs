//! Search results controller.
//!
//! Three stages, mirroring `album.rs`: `load_search` fetches a combined
//! search through `QbzCore` on a worker thread, `map_*` turns the domain
//! types into plain `Send` rows (the unit-tested layer), and
//! `apply_search` writes the `SearchState` global on the Slint event loop.

mod apply;
mod artwork;
mod cortinilla;
mod load;
mod local_rows;
mod mappers;
mod pagination;
mod pure;
mod rows;
mod version;

// Re-export the full public surface so `crate::search::X` paths are
// unchanged for every caller across the `qbz` crate.
pub use apply::{
    apply_cortinilla, apply_search, cortinilla_artwork_jobs, mark_artist_followed, recompute_hi_res_filtered,
    reset_search,
};
pub(crate) use apply::playlist_item;
pub use artwork::{artwork_jobs, artwork_jobs_for_more};
pub use load::{load_cortinilla, load_search};
pub use mappers::map_playlist;
pub use pagination::{append_results, category_for_tab, load_more, replace_category, search_type_for_filter, SearchCategory};
pub use rows::{
    CortRow, CortinillaData, PlaylistRow,
};
pub use version::{
    is_current_cortinilla_version, is_current_version,
    next_cortinilla_version, next_search_version,
};
