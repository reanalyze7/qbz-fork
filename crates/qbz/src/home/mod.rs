//! Discover / Home controller.
//!
//! Fetches the Qobuz discover index through `QbzCore`, maps it into plain
//! (Send) data on the worker thread, and — separately, on the Slint event
//! loop — converts that into Slint models pushed onto the `HomeState`
//! global. Domain types never reach the `.slint` files.

use std::cell::RefCell;

mod descriptors;
mod load;
mod map;
mod present;
mod tags;
#[cfg(test)]
mod tests;
mod types;

pub use descriptors::{rerender_active_tab, select_tab, tab_descriptors};
pub use load::{load_home, recent_album_cards, recent_track_slims};
pub(crate) use map::{map_album, map_playlist};
pub use present::{apply_home, apply_recent_rails, playlist_artwork_jobs};
pub(crate) use present::{card_to_item, playlist_to_item};
pub use tags::{clear_playlist_tags, toggle_playlist_tag};
pub use types::{CardData, HomeData, PlaylistCardData, SectionData, SlimData};

thread_local! {
    /// The per-tab section sets, cached on the UI thread after a load
    /// so a tab switch can swap HomeState.sections without re-fetching.
    /// (home, editor, foryou)
    pub(super) static TAB_SECTIONS: RefCell<TabSections> = RefCell::new(TabSections::default());
}

#[derive(Default)]
pub(super) struct TabSections {
    pub(super) home: Vec<SectionData>,
    pub(super) editor: Vec<SectionData>,
    pub(super) home_playlists: Vec<PlaylistCardData>,
    pub(super) editor_playlists: Vec<PlaylistCardData>,
    /// Slugs of the currently-selected category tags (Qobuz Playlists filter).
    /// Empty = show all. Client-side; survives a tab switch.
    pub(super) selected_tags: Vec<String>,
}

/// Keep only the playlists whose tag slugs intersect `selected` (union of the
/// selected tags). An empty selection passes everything through.
pub(super) fn filter_playlists<'a>(
    playlists: &'a [PlaylistCardData],
    selected: &[String],
) -> Vec<&'a PlaylistCardData> {
    if selected.is_empty() {
        return playlists.iter().collect();
    }
    playlists
        .iter()
        .filter(|p| p.tags.iter().any(|slug| selected.iter().any(|s| s == slug)))
        .collect()
}

