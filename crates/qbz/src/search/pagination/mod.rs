//! Load-more pagination for the search results page.

mod apply;
mod load;

pub use apply::{append_results, replace_category};
pub use load::load_more;

use crate::search::rows::{AlbumRow, ArtistRow, PlaylistRow, TrackRowData};

/// Which category a load-more request targets.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SearchCategory {
    Albums,
    Tracks,
    Artists,
    Playlists,
}

/// Map a results-tab index to the category whose list it paginates.
/// Tab 0 (All) has no single category.
pub fn category_for_tab(tab: i32) -> Option<SearchCategory> {
    match tab {
        1 => Some(SearchCategory::Albums),
        2 => Some(SearchCategory::Tracks),
        3 => Some(SearchCategory::Artists),
        4 => Some(SearchCategory::Playlists),
        _ => None,
    }
}

/// Map a filter index to the Qobuz `search_type` value. Index 0 maps to
/// `None` (no filter).
pub fn search_type_for_filter(index: i32) -> Option<String> {
    match index {
        1 => Some("MainArtist".into()),
        2 => Some("Performer".into()),
        3 => Some("Composer".into()),
        4 => Some("Label".into()),
        5 => Some("ReleaseName".into()),
        _ => None,
    }
}

/// A page of additional rows fetched by load-more, ready to append.
pub enum MoreRows {
    Albums(Vec<AlbumRow>),
    Tracks(Vec<TrackRowData>),
    Artists(Vec<ArtistRow>),
    Playlists(Vec<PlaylistRow>),
}

/// Load-more page size (matches the Tauri search page size).
pub(crate) const PAGE_SIZE: u32 = 20;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_for_tab_maps_per_type_tabs() {
        assert_eq!(category_for_tab(0), None);
        assert_eq!(category_for_tab(1), Some(SearchCategory::Albums));
        assert_eq!(category_for_tab(2), Some(SearchCategory::Tracks));
        assert_eq!(category_for_tab(3), Some(SearchCategory::Artists));
        assert_eq!(category_for_tab(4), Some(SearchCategory::Playlists));
        assert_eq!(category_for_tab(9), None);
    }

    #[test]
    fn search_type_for_filter_maps_dropdown_index() {
        assert_eq!(search_type_for_filter(0), None);
        assert_eq!(search_type_for_filter(1), Some("MainArtist".to_string()));
        assert_eq!(search_type_for_filter(3), Some("Composer".to_string()));
        assert_eq!(search_type_for_filter(5), Some("ReleaseName".to_string()));
        assert_eq!(search_type_for_filter(99), None);
    }
}
