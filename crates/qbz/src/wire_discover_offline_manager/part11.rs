use crate::*;

pub(crate) fn wire_discover_offline_manager_part11(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {

    // Header nav-menu navigation — currently routes the Library
    // dropdown rows into Library > Favorites tabs.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window.on_header_menu_navigate(move |route| {
            if route == "home" {
                if let Some(w) = weak.upgrade() {
                    w.global::<NavState>().set_view(ContentView::Home);
                }
                return;
            }
            // My QBZ — Mixtapes / Collections index grids (read-only slice).
            // Record history + navigate (loads via myqbz::navigate), mirroring
            // the Favorites / Local Library per-route pattern.
            if route == "myqbz-mixtapes" {
                nav::record(nav::NavEntry::Mixtapes);
                if let Some(w) = weak.upgrade() {
                    update_nav_flags(&w);
                }
                myqbz::navigate(
                    weak.clone(),
                    handle.clone(),
                    image_cache.clone(),
                    qbz_models::mixtape::CollectionKind::Mixtape,
                );
                return;
            }
            if route == "myqbz-collections" {
                nav::record(nav::NavEntry::Collections);
                if let Some(w) = weak.upgrade() {
                    update_nav_flags(&w);
                }
                myqbz::navigate(
                    weak.clone(),
                    handle.clone(),
                    image_cache.clone(),
                    qbz_models::mixtape::CollectionKind::Collection,
                );
                return;
            }
            // Discover tabs — switch to Home and select the tab. The
            // section sets are already cached from the initial load,
            // so this just swaps the visible set + re-fires artwork.
            if let Some(tab) = route.strip_prefix("discover-") {
                let tab = tab.to_string();
                if let Some(w) = weak.upgrade() {
                    nav::record(nav::NavEntry::Discover { tab: tab.clone() });
                    w.global::<NavState>().set_view(ContentView::Home);
                    let jobs = home::select_tab(&w, &tab);
                    artwork::spawn_loads(jobs, weak.clone(), image_cache.clone());
                    update_nav_flags(&w);
                    if tab == "forYou" {
                        ensure_for_you_loaded(&runtime, &weak, &handle, &image_cache);
                    }
                    if tab == "recommendations" {
                        external_reco::ensure_loaded(&runtime, &weak, &handle, &image_cache);
                    }
                }
                return;
            }
            if route.as_str() == "favorites-all" {
                nav::record(nav::NavEntry::Favorites {
                    tab: "all".to_string(),
                });
                if let Some(w) = weak.upgrade() {
                    update_nav_flags(&w);
                }
                navigate_library_all(
                    runtime.clone(),
                    weak.clone(),
                    &handle,
                    image_cache.clone(),
                );
                return;
            }
            if let Some(tab) = favorites::FavTab::from_route(route.as_str()) {
                let tab_id = route.strip_prefix("favorites-").unwrap_or("tracks");
                nav::record(nav::NavEntry::Favorites {
                    tab: tab_id.to_string(),
                });
                if let Some(w) = weak.upgrade() {
                    update_nav_flags(&w);
                }
                navigate_favorites(
                    runtime.clone(),
                    weak.clone(),
                    &handle,
                    image_cache.clone(),
                    tab,
                    tab_id,
                );
                return;
            }
            // Local Library tabs — same per-tab history pattern as Favorites.
            if let Some(tab) = local_library::LibTab::from_route(route.as_str()) {
                nav::record(nav::NavEntry::LocalLibrary {
                    tab: tab.tab_id().to_string(),
                });
                if let Some(w) = weak.upgrade() {
                    update_nav_flags(&w);
                }
                navigate_local_library(
                    runtime.clone(),
                    weak.clone(),
                    &handle,
                    image_cache.clone(),
                    tab,
                );
            }
        });
    }
}
