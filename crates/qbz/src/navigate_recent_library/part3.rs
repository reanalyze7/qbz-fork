use crate::*;

/// Reorder the cached Library-Albums base list per the sort selection.
/// `mode`: 0 = Recently added (base), 1 = First added (reverse), 2 = Random.
pub(crate) fn library_albums_sorted(mode: i32) -> Vec<crate::foryou::AlbumCard> {
    let mut cards = LIB_ALBUMS_BASE.lock().unwrap().clone();
    match mode {
        1 => cards.reverse(),
        2 => {
            use rand::seq::SliceRandom;
            cards.shuffle(&mut rand::rng());
        }
        _ => {}
    }
    cards
}

/// Re-push the Home "Library Albums" rail in the chosen sort order + re-fire
/// its artwork by the new index. No re-fetch — reorders the cached base list
/// (mirrors `refresh_recent_rails`' local re-apply). Runs on the UI thread
/// (Slint callback); `spawn_loads` does its own threading and the covers are
/// cache-served, so a reorder is near-instant.
pub(crate) fn apply_library_albums_sort(
    weak: slint::Weak<AppWindow>,
    mode: i32,
    image_cache: artwork::ImageCache,
) {
    let cards = library_albums_sorted(mode);
    if let Some(w) = weak.upgrade() {
        w.global::<HomeState>().set_favorite_albums(crate::foryou::section(
            &qbz_i18n::t("Library Albums"),
            &cards,
        ));
    }
    let jobs: Vec<artwork::ArtworkJob> = cards
        .iter()
        .enumerate()
        .filter_map(|(idx, card)| {
            (!card.artwork_url.is_empty()).then(|| artwork::ArtworkJob {
                target: artwork::ArtworkTarget::HomeFavoriteAlbum { idx },
                url: card.artwork_url.clone(),
            })
        })
        .collect();
    artwork::spawn_loads(jobs, weak, image_cache);
}

/// Playback hook: a play was just recorded in the recently-played store
/// (`playback::record_recent`). Marks the Home rails stale and — when the
/// Home view is currently showing — refreshes them immediately, so the rails
/// track the session live instead of only filling on restart. Off-Home
/// nothing is read; the dirty flag makes the next Home mount refresh. Event-
/// driven only — no timer, no polling.
pub(crate) fn note_recent_store_changed(
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
) {
    RECENT_RAILS_DIRTY.store(true, std::sync::atomic::Ordering::Relaxed);
    let Some(image_cache) = artwork::shared_cache() else {
        return;
    };
    // The visible-view check must read NavState on the UI thread.
    let _ = weak.clone().upgrade_in_event_loop(move |w| {
        if w.global::<NavState>().get_view() == ContentView::Home {
            refresh_recent_rails(weak, &handle, image_cache);
        }
    });
}

/// Open Library > Favorites on `tab` and lazy-load that tab's data.
/// Switching the active tab also routes here so each tab fetches on
/// first view (Tauri's loadTabIfNeeded).
pub(crate) fn navigate_favorites(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: artwork::ImageCache,
    tab: favorites::FavTab,
    tab_id: &str,
) {
    let tab_id = tab_id.to_string();
    handle.spawn(async move {
        let tab_id_for_ui = tab_id.clone();
        let _ = weak.upgrade_in_event_loop(move |w| {
            let state = w.global::<FavoritesState>();
            state.set_active_tab(tab_id_for_ui.into());
            favorites::reset_loading(&w);
            w.global::<NavState>().set_view(ContentView::Favorites);
            // The genre popup edits the favorites context here, and the
            // toolbar genre button shows the favorites selection count.
            genre_filter::set_context("favorites");
            genre_filter::apply_state(&w);
            // Restore persisted toolbar choices before the data applies +
            // derives, so the loaded view honors them.
            favorites_prefs::load(&w);
        });
        match favorites::load_favorites(&runtime, tab).await {
            Ok(data) => {
                let jobs = favorites::artwork_jobs(&data);
                // WINDOWED artwork for the Albums tab (was: a job for every
                // favorite album). Reset the pipeline BEFORE apply — apply's
                // `derive_albums` dispatches the covers itself (flat grid =
                // viewport band; list/grouped = full set). Other tabs keep
                // the all-at-once `jobs` dispatch.
                let is_albums = matches!(&data, favorites::FavData::Albums { .. });
                let image_cache_for_ui = image_cache.clone();
                let _ = weak.upgrade_in_event_loop(move |w| {
                    if is_albums {
                        favorites::begin_albums_artwork(image_cache_for_ui);
                    }
                    favorites::apply_favorites(&w, data);
                });
                artwork::spawn_loads(jobs, weak.clone(), image_cache.clone());
            }
            Err(e) => {
                log::error!("[qbz-slint] favorites load failed: {e}");
                let msg = e.to_string();
                let _ = weak.upgrade_in_event_loop(move |w| {
                    let st = w.global::<FavoritesState>();
                    st.set_loading(false);
                    st.set_load_error(msg.into());
                });
            }
        }
    });
}

