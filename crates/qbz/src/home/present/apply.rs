//! Push worker-thread home data (and the targeted recent-rails refresh)
//! onto the `HomeState` Slint global. Must run on the Slint event loop.

use slint::ComponentHandle;
use slint::ModelRc;
use slint::VecModel;

use crate::{AlbumCardItem, AppWindow, HomeState, PlaylistTagItem, SearchPlaylistItem, SlimItem};

use super::super::{CardData, HomeData, SlimData, TabSections, TAB_SECTIONS};
use super::items::{card_to_item, playlist_to_item, slim_to_item};
use super::sections::build_sections;

/// Push ONLY the two recently-played rails onto `HomeState` — the targeted
/// auto/manual refresh path. Everything else on Home is left untouched: no
/// discover-index fetch, no descriptor rebuild, no section-cache write. Must
/// run on the Slint event loop (`card_to_item` seeds is-favorite from the
/// login cache, same as `apply_home`).
pub fn apply_recent_rails(window: &AppWindow, recent: Vec<SlimData>, albums: Vec<CardData>) {
    let recent: Vec<SlimItem> = recent.into_iter().map(slim_to_item).collect();
    let albums: Vec<AlbumCardItem> = albums.into_iter().map(card_to_item).collect();
    let state = window.global::<HomeState>();
    state.set_recent(ModelRc::new(VecModel::from(recent)));
    state.set_recent_albums(ModelRc::new(VecModel::from(albums)));
}

/// Convert worker-thread home data into Slint models and push them onto
/// the `HomeState` global. Must run on the Slint event loop.
pub fn apply_home(window: &AppWindow, data: HomeData) {
    let sections = build_sections(&data.sections);

    // Cache the Home + Editor's Picks section sets for instant tab
    // switching (For You has its own dedicated state/view). A fresh index
    // load resets the category-tag selection (the tag set may have changed).
    TAB_SECTIONS.with(|cell| {
        *cell.borrow_mut() = TabSections {
            home: data.sections.clone(),
            editor: data.editor_sections.clone(),
            home_playlists: data.playlists.clone(),
            editor_playlists: data.editor_playlists.clone(),
            selected_tags: Vec::new(),
        };
    });

    let to_slim_items =
        |items: Vec<SlimData>| -> Vec<SlimItem> { items.into_iter().map(slim_to_item).collect() };
    let popular = to_slim_items(data.popular);
    let recent = to_slim_items(data.recent);
    let recent_albums: Vec<AlbumCardItem> =
        data.recent_albums.into_iter().map(card_to_item).collect();

    // Push the HOME tab's Qobuz Playlists row (apply_home runs for the
    // default Home tab; a tab switch swaps it via select_tab). The selection
    // was just reset, so the unfiltered full set is shown.
    let home_playlists: Vec<SearchPlaylistItem> =
        data.playlists.iter().map(playlist_to_item).collect();
    // Category tags for the multi-select filter — all start unselected.
    let tag_items: Vec<PlaylistTagItem> = data
        .playlist_tags
        .iter()
        .map(|(slug, name)| PlaylistTagItem {
            slug: slug.clone().into(),
            name: name.clone().into(),
            selected: false,
        })
        .collect();

    let state = window.global::<HomeState>();
    state.set_sections(ModelRc::new(VecModel::from(sections)));
    state.set_popular(ModelRc::new(VecModel::from(popular)));
    state.set_recent(ModelRc::new(VecModel::from(recent)));
    state.set_recent_albums(ModelRc::new(VecModel::from(recent_albums)));
    // The #566 ported rails — same section builders + title msgids as their
    // For You twins (foryou::apply_favorite_albums / apply_release_watch /
    // apply_top_artists), separate lifecycles. Top Artists' title lives in
    // the HomeView arm (@tr, like ForYouView's) — its model is a bare list.
    // Cache the base (Recently-added) order so the header sort dropdown can
    // reorder without a re-fetch, and reset the selection to the default on
    // every fresh load (the load-time artwork dispatch is in base order).
    *crate::LIB_ALBUMS_BASE.lock().unwrap() = data.favorite_albums.clone();
    state.set_library_albums_sort(0);
    state.set_favorite_albums(crate::foryou::section(
        &qbz_i18n::t("Library Albums"),
        &data.favorite_albums,
    ));
    state.set_release_watch(crate::foryou::section(
        &qbz_i18n::t("Release Watch"),
        &data.release_watch,
    ));
    state.set_most_played_albums(crate::foryou::section(
        &qbz_i18n::t("Most Played Albums"),
        &data.most_played_albums,
    ));
    state.set_top_artists(ModelRc::new(VecModel::from(crate::foryou::artist_items(
        &data.top_artists,
    ))));
    state.set_playlists(ModelRc::new(VecModel::from(home_playlists)));
    state.set_playlist_tags(ModelRc::new(VecModel::from(tag_items)));
    state.set_playlist_tag_count(0);

    // Push the prefs-driven descriptor lists now that the section cache is
    // populated, so the Home/Editor render loop reflects the fresh data (the
    // following select_tab re-pushes for the active tab; this keeps the lists
    // correct even if select_tab is not called).
    let prefs = crate::discover_prefs::prefs_snapshot();
    crate::discover_prefs::push_descriptors(window, &prefs);
}
