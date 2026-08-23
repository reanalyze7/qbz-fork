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
    apply_cortinilla, apply_immersive_search, apply_search, cortinilla_artwork_jobs,
    immersive_cortinilla_artwork_jobs, mark_artist_followed, recompute_hi_res_filtered,
    reset_search,
};
pub(crate) use apply::playlist_item;
pub use artwork::{artwork_jobs, artwork_jobs_for_more};
pub use cortinilla::{map_search_all_to_cortinilla, map_search_all_to_immersive};
pub use load::{load_cortinilla, load_immersive_search, load_search};
pub use local_rows::{append_local_sections, load_cortinilla_local, LocalCaps};
pub use mappers::{map_album, map_artist, map_playlist, map_search_all, map_track};
pub use pagination::{append_results, category_for_tab, load_more, replace_category, search_type_for_filter, MoreRows, SearchCategory};
pub use rows::{
    AlbumRow, ArtistRow, CortRow, CortSection, CortinillaData, MostPopularRow, PlaylistRow,
    SearchData, TrackRowData,
};
pub use version::{
    is_current_cortinilla_version, is_current_immersive_search_version, is_current_version,
    next_cortinilla_version, next_immersive_search_version, next_search_version,
};
