use crate::*;

/// Flip `is-following` on every visible PLAYLIST card matching `playlist_id`
/// (Home rail, Qobuz Playlists browse, Search, Favorites, label/artist
/// landings, Pinned carousel) so a follow/unfollow from ANY card overlay or
/// menu updates the others live. Playlist twin of [`set_artist_row_pinned`].
pub(crate) fn set_playlist_row_following(window: &AppWindow, playlist_id: &str, following: bool) {
    let flip = |model: &slint::ModelRc<SearchPlaylistItem>| {
        for i in 0..model.row_count() {
            if let Some(mut item) = model.row_data(i) {
                if item.id == playlist_id && item.is_following != following {
                    item.is_following = following;
                    model.set_row_data(i, item);
                }
            }
        }
    };
    flip(&window.global::<HomeState>().get_playlists());
    flip(&window.global::<SearchState>().get_playlists());
    let browse = window.global::<PlaylistBrowseState>();
    flip(&browse.get_playlists());
    flip(&browse.get_visible());
    let favs = window.global::<FavoritesState>();
    flip(&favs.get_playlists_favorites());
    flip(&favs.get_playlists_following());
    flip(&favs.get_playlists_visible());
    flip(&window.global::<LabelState>().get_playlists());
    flip(&window.global::<ArtistState>().get_playlists());
    // Pinned mixed carousel — nested SearchPlaylistItem.
    let pm = window.global::<PinnedState>().get_items();
    for i in 0..pm.row_count() {
        if let Some(mut it) = pm.row_data(i) {
            if it.kind == "playlist"
                && it.playlist.id == playlist_id
                && it.playlist.is_following != following
            {
                it.playlist.is_following = following;
                pm.set_row_data(i, it);
            }
        }
    }
}

/// Feed Capa B (intelligent-search ranking) from a RESULTS-PAGE click, but only
/// when the results page is the active view. `on_open_album` / `on_open_artist`
/// / `on_media_action` are global handlers shared by every view (album detail,
/// home, favorites, …); without this gate a play/open from any of those would
/// be misattributed to the current search query. The `SearchState.query`
/// (results-page query, distinct from the live `cortinilla-query`) is the key.
///
/// No-op when the active view is not Search, when the module is disabled (the
/// `record` accessor itself no-ops then too), or when the query is empty. LOCAL
/// entities are NOT routed here — local rows never reach the Qobuz results page
/// (D1/D2) and use a different id space (D4).
pub(crate) fn record_search_interaction(
    window: &AppWindow,
    kind: &str,
    id: &str,
    action: crate::search_service::InteractionAction,
) {
    if window.global::<NavState>().get_view() != ContentView::Search {
        return;
    }
    if !crate::search_service::is_enabled() {
        return;
    }
    let query = window.global::<SearchState>().get_query().to_string();
    if query.trim().is_empty() {
        return;
    }
    crate::search_service::record(&query, kind, id, action);
}

