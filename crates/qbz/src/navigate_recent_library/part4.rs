use crate::*;

/// Navigate to the Library "All" mixed feed (webplayer /user-library/all).
/// Fans out to every source, merges + orders by date added, then applies +
/// dispatches cover artwork. Rendered by the FavoritesView `active-tab == "all"`
/// branch reading `LibraryAllState`.
pub(crate) fn navigate_library_all(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: artwork::ImageCache,
) {
    handle.spawn(async move {
        let _ = weak.upgrade_in_event_loop(|w| {
            w.global::<FavoritesState>().set_active_tab("all".into());
            w.global::<NavState>().set_view(ContentView::Favorites);
            let st = w.global::<LibraryAllState>();
            st.set_loading(true);
            st.set_load_error("".into());
            // The mixed feed has its OWN genre-filter context (independent of
            // the favorites albums/tracks filter) so the toolbar badge reflects
            // this surface's selection on entry.
            genre_filter::set_context("library-all");
            genre_filter::apply_state(&w);
        });
        match library_all::load_library_all(&runtime).await {
            Ok(feed) => {
                let weak_j = weak.clone();
                let ic = image_cache.clone();
                let _ = weak.upgrade_in_event_loop(move |w| {
                    library_all::apply_library_all(&w, feed);
                    let jobs = library_all::artwork_jobs(&w);
                    // Mixed payload (Qobuz http / local fs) — route each
                    // cover by scheme so local covers decode.
                    artwork::spawn_search_loads(jobs, weak_j.clone(), ic.clone());
                });
            }
            Err(e) => {
                log::error!("[qbz-slint] library-all load failed: {e}");
                let msg = e.to_string();
                let _ = weak.upgrade_in_event_loop(move |w| {
                    let st = w.global::<LibraryAllState>();
                    st.set_loading(false);
                    st.set_load_error(msg.into());
                });
            }
        }
    });
}

/// Navigate to the LocalLibrary Artists tab and auto-select `name`. Local
/// artists have no id — they're keyed by NAME. The selection is latched and
/// consumed by `ensure_artists_loaded` once the tab's data is ready (handles
/// both the already-loaded and still-loading cases). Used by the LocalAlbum
/// header artist link, the now-playing "Go to artist", and local track menus.
pub(crate) fn open_local_artist(
    runtime: &Arc<AppRuntime<SlintAdapter>>,
    weak: &slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: &artwork::ImageCache,
    name: String,
) {
    if name.trim().is_empty() {
        return;
    }
    local_library::set_pending_artist(name);
    nav::record(nav::NavEntry::LocalLibrary {
        tab: "artists".to_string(),
    });
    navigate_local_library(
        runtime.clone(),
        weak.clone(),
        handle,
        image_cache.clone(),
        local_library::LibTab::Artists,
    );
    if let Some(w) = weak.upgrade() {
        update_nav_flags(&w);
    }
}

