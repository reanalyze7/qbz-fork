use crate::*;

/// Album counterpart of [`set_row_favorite`]: flip the `is-favorite` heart on
/// every visible album CARD matching `album_id`, across all card surfaces
/// (artist discography, album-detail carousels, home/discover/for-you rows,
/// search, label/award grids, favorites). Cards read `fav_cache` when they are
/// (re)built; this keeps the ones already on screen in sync the instant a
/// favorite is added or removed anywhere (album header heart, card heart,
/// favorites-view unfavorite).
pub(crate) fn set_album_row_favorite(window: &AppWindow, album_id: &str, favorite: bool) {
    let flip = |model: &slint::ModelRc<AlbumCardItem>| {
        for i in 0..model.row_count() {
            if let Some(mut item) = model.row_data(i) {
                if item.id == album_id && item.is_favorite != favorite {
                    item.is_favorite = favorite;
                    model.set_row_data(i, item);
                }
            }
        }
    };
    let flip_section = |section: &DiscoverSection| flip(&section.albums);
    let flip_sections = |model: &slint::ModelRc<DiscoverSection>| {
        for s in 0..model.row_count() {
            if let Some(section) = model.row_data(s) {
                flip(&section.albums);
            }
        }
    };

    // Artist page — release sections + last-release + the in-page-search
    // FULL cache (owned by artist.rs).
    artist::set_release_card_favorite(window, album_id, favorite);
    // Dedicated discography page (View all).
    flip(&window.global::<ArtistReleasesState>().get_albums());
    // Album detail carousels — From the same artist / Listening suggestions.
    let album = window.global::<AlbumState>();
    flip_section(&album.get_more_from_artist());
    flip_section(&album.get_suggestions_section());
    flip_section(&album.get_lastfm_suggestions_section());
    // Search results + the most-popular album hero.
    let search = window.global::<SearchState>();
    flip(&search.get_albums());
    let mut hero = search.get_most_popular_album();
    if hero.id == album_id && hero.is_favorite != favorite {
        hero.is_favorite = favorite;
        search.set_most_popular_album(hero);
    }
    // Home / Editor's Picks — the descriptor-driven carousels render the
    // page; HomeState.sections + recent-albums back the fixed-data arms.
    let home = window.global::<HomeState>();
    flip_sections(&home.get_sections());
    flip(&home.get_recent_albums());
    let discover = window.global::<DiscoverState>();
    for model in [
        discover.get_home_sections(),
        discover.get_editor_sections(),
        discover.get_foryou_sections(),
    ] {
        for s in 0..model.row_count() {
            if let Some(desc) = model.row_data(s) {
                flip(&desc.section.albums);
            }
        }
    }
    // Discover "View all" page.
    let browse = window.global::<DiscoverBrowseState>();
    flip(&browse.get_albums());
    flip(&browse.get_visible());
    // For You.
    let foryou = window.global::<ForYouState>();
    flip_section(&foryou.get_release_watch());
    flip_section(&foryou.get_recent_albums());
    flip_section(&foryou.get_favorite_albums());
    flip_section(&foryou.get_more_from_library());
    flip_section(&foryou.get_rediscover());
    flip(&foryou.get_spotlight_albums());
    // Recommendations (external reco).
    let reco = window.global::<ExternalRecoState>();
    flip_section(&reco.get_rec_albums());
    flip_section(&reco.get_fresh_releases());
    flip_section(&reco.get_deep_cut_albums());
    flip_section(&reco.get_top_albums());
    // Label pages (landing carousels + releases grid).
    let label = window.global::<LabelState>();
    flip(&label.get_albums());
    flip(&label.get_visible());
    flip_sections(&label.get_grouped());
    flip_section(&label.get_releases_section());
    flip_section(&label.get_critics_section());
    // Favorites — albums tab (flat + grouped) and the artists sidepanel.
    let favs = window.global::<FavoritesState>();
    flip(&favs.get_albums());
    flip(&favs.get_albums_visible());
    flip_sections(&favs.get_albums_grouped());
    flip_sections(&favs.get_selected_artist_sections());
    // Dedicated "Recently played — view all" page.
    flip(&window.global::<RecentAlbumsState>().get_albums());
    // Pinned mixed carousel (Home / For You) — the album lives NESTED inside a
    // PinnedItem, so the generic `flip` can't reach it.
    set_pinned_album_favorite(window, album_id, favorite);
    // Now-playing bar "+" flyout — its add/remove-album-to-collection entry
    // reads NowPlayingState.album-favorite; flip it when the toggled album is
    // the one playing so the label stays honest without a track change.
    // (Seeded per-track from fav_cache in playback::refresh_now_playing_meta.)
    let np = window.global::<NowPlayingState>();
    if np.get_album_id() == album_id {
        np.set_album_favorite(favorite);
    }
    // Album-detail HEADER heart: without this, a toggle from any other
    // surface (cards, NPB flyout) leaves the open album page's heart stale
    // and its next click silently UNDOES the user's action (the toggle
    // reads the already-flipped cache). Redundant-but-harmless when the
    // header arm itself called us — same value.
    let album_state = window.global::<AlbumState>();
    if album_state.get_id() == album_id {
        album_state.set_is_favorite(favorite);
    }
}

