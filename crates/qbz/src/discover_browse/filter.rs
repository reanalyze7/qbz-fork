//! `apply_filter`: rebuild the visible list from the current search query.

use slint::{ComponentHandle, Model, ModelRc, VecModel};

use crate::{AlbumCardItem, AppWindow, DiscoverBrowseState};

/// Rebuild `visible` from `albums` honoring the current search query
/// (case-insensitive substring over title + artist). UI thread only.
/// Wired to DiscoverBrowseActions::search-changed and called after every
/// model mutation so the rendered list stays consistent.
pub fn apply_filter(window: &AppWindow) {
    let state = window.global::<DiscoverBrowseState>();
    let query = state.get_search_query().trim().to_lowercase();
    let albums = state.get_albums();
    if query.is_empty() {
        // No filter — share the SAME model so artwork-pipeline updates
        // (which mutate `albums[index]`) propagate to the rendered list
        // without rebuilding it. This is the common case.
        state.set_visible(albums);
        return;
    }
    let visible: Vec<AlbumCardItem> = (0..albums.row_count())
        .filter_map(|i| albums.row_data(i))
        .filter(|a| {
            a.title.to_lowercase().contains(&query)
                || a.artist.to_lowercase().contains(&query)
        })
        .collect();
    state.set_visible(ModelRc::new(VecModel::from(visible)));
}
