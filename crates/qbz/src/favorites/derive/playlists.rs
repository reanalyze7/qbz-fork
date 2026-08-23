use slint::{ComponentHandle, Model, ModelRc, VecModel};

use crate::{AppWindow, FavoritesState, SearchPlaylistItem};

/// Re-derive the rendered Playlists grid/list (`playlists-visible`) from the
/// active sub-tab source (`playlists-favorites` / `playlists-following`) and
/// the search query. Empty query shares the source model so collage artwork
/// stays live; a query forks a name/owner-filtered clone (mirrors Tauri's
/// filteredPlaylists, which matches name OR owner — the owner is part of the
/// item subtitle). No sort, no group (Tauri has none).
pub fn derive_playlists(window: &AppWindow) {
    let state = window.global::<FavoritesState>();
    let source = if state.get_playlists_sub_tab().as_str() == "following" {
        state.get_playlists_following()
    } else {
        state.get_playlists_favorites()
    };
    let query_owned = state.get_playlists_search().to_lowercase();
    let query = query_owned.trim();
    if query.is_empty() {
        state.set_playlists_visible(source);
        return;
    }
    let filtered: Vec<SearchPlaylistItem> = (0..source.row_count())
        .filter_map(|i| source.row_data(i))
        .filter(|p| {
            p.title.to_lowercase().contains(query) || p.subtitle.to_lowercase().contains(query)
        })
        .collect();
    state.set_playlists_visible(ModelRc::new(VecModel::from(filtered)));
}
