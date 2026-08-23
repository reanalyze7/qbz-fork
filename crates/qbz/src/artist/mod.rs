//! Artist detail controller.
//!
//! Fetches an artist page through `QbzCore`, maps it to plain (Send)
//! data on the worker thread, and applies it to the `ArtistState`
//! global on the Slint event loop.

mod apply;
mod apply_sections;
mod artwork_apply;
mod artwork_jobs;
mod cache;
mod data;
mod favorites;
mod jump_tabs;
mod load;
mod mb;
mod multi_select;
mod page_append;
mod search;
mod sort_page;
mod stories;
mod track_map;

// Re-export the full public surface so `crate::artist::X` paths are
// unchanged for every caller across the `qbz` crate.
pub use apply::apply_artist;
pub use artwork_apply::apply_artwork;
pub use artwork_jobs::artwork_jobs;
pub use cache::MAX_INDEX_PAGES;
pub use data::{
    release_type_title, ArtistData, LabelData, PlaylistSlim, ReleaseSection, SimilarArtistData,
};
pub use favorites::{reset_artist, set_release_card_favorite, set_release_card_pinned};
pub use load::{load_artist, load_release_page, RELEASE_PAGE_SIZE};
pub use mb::{
    apply_mb_discovery, apply_mb_metadata, apply_mb_relationships, apply_mb_unavailable,
    load_mb_discovery, load_mb_metadata, load_mb_relationships, location_params,
    remove_discovery_artist, reset_network_sidebar, LocationParams, MbDiscoveryData,
    MbDiscoveryRow, MbMetadata, MbOrigin, MbRelationshipRow, MbRelationshipsRowData,
};
pub use multi_select::{
    all_top_track_ids, clear_selection, recount_selected, select_all, selected_ids,
    set_multi_select,
};
pub use page_append::append_release_page;
pub use search::filter_artist;
pub use sort_page::{resort_section, section_can_load_more, section_loaded_count};
pub use stories::{apply_stories, load_stories, StoryData};
pub(crate) use track_map::{card_to_item, map_release};
