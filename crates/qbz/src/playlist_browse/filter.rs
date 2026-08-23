//! Client-side search-query filter over the loaded set.

use slint::{ComponentHandle, Model, ModelRc, VecModel};

use crate::{AppWindow, PlaylistBrowseState, SearchPlaylistItem};

/// Rebuild `visible` from `playlists` honoring the current search query
/// (case-insensitive substring over title + subtitle). UI thread only.
/// Wired to PlaylistBrowseActions::search-changed and called after every
/// model mutation so the rendered list stays consistent.
pub fn apply_filter(window: &AppWindow) {
    let state = window.global::<PlaylistBrowseState>();
    let query = state.get_search_query().trim().to_lowercase();
    let playlists = state.get_playlists();
    if query.is_empty() {
        // No filter — share the SAME model so artwork-pipeline updates
        // (which mutate `playlists[idx]`) propagate to the rendered list
        // without rebuilding it. This is the common case.
        state.set_visible(playlists);
        return;
    }
    let visible: Vec<SearchPlaylistItem> = (0..playlists.row_count())
        .filter_map(|i| playlists.row_data(i))
        .filter(|p| {
            p.title.to_lowercase().contains(&query)
                || p.subtitle.to_lowercase().contains(&query)
        })
        .collect();
    state.set_visible(ModelRc::new(VecModel::from(visible)));
}
