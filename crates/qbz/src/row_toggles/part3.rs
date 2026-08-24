use crate::*;

/// Pin twin of [`set_album_row_favorite`]: flip the `is-pinned` badge on every
/// visible album CARD matching `album_id`, across all card surfaces. Cards
/// read `crate::pinned` when they are (re)built; this keeps the ones already
/// on screen in sync the instant a pin toggles anywhere. The Pinned section's
/// own model is NOT walked — `pinned_section::rebuild_pinned` replaces it
/// wholesale right after. Unlike the favorite twin this also walks the Local
/// Library models: LL cards hide the heart but do show the pin glyph.
pub(crate) fn set_album_row_pinned(window: &AppWindow, album_id: &str, pinned: bool) {
    let flip = |model: &slint::ModelRc<AlbumCardItem>| {
        for i in 0..model.row_count() {
            if let Some(mut item) = model.row_data(i) {
                if item.id == album_id && item.is_pinned != pinned {
                    item.is_pinned = pinned;
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
    artist::set_release_card_pinned(window, album_id, pinned);
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
    if hero.id == album_id && hero.is_pinned != pinned {
        hero.is_pinned = pinned;
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
    // Local Library — albums + folders tabs (flat, visible and grouped) and
    // the artists tab's selected-artist grid. The pin glyph is live here even
    // though the favorite heart is hidden.
    let ll = window.global::<LocalLibraryState>();
    flip(&ll.get_albums());
    flip(&ll.get_albums_visible());
    flip_sections(&ll.get_albums_grouped());
    flip(&ll.get_folders());
    flip(&ll.get_folders_visible());
    flip_sections(&ll.get_folders_grouped());
    flip(&ll.get_artists_selected_albums());
    // Dedicated "Recently played — view all" page (the Pinned carousel itself is
    // rebuilt wholesale by pinned_section::rebuild_pinned right after a pin).
    flip(&window.global::<RecentAlbumsState>().get_albums());
}

