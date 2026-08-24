use crate::*;

pub(crate) fn wire_library_all_artwork_close_part1(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        let weak = window.as_weak();
        let image_cache = image_cache.clone();
        window.global::<LibraryAllActions>().on_set_sort(move |key| {
            if let Some(w) = weak.upgrade() {
                // Re-selecting the active field toggles asc/desc (PlaylistView
                // pattern); a new field resets to its natural direction.
                library_all::set_sort(&w, key.as_str());
                let jobs = library_all::artwork_jobs(&w);
                artwork::spawn_search_loads(jobs, weak.clone(), image_cache.clone());
            }
        });
    }
    {
        let weak = window.as_weak();
        window.global::<LibraryAllActions>().on_set_view(move |mode| {
            if let Some(w) = weak.upgrade() {
                w.global::<LibraryAllState>().set_view_mode(mode);
            }
        });
    }
    {
        let weak = window.as_weak();
        let image_cache = image_cache.clone();
        window
            .global::<LibraryAllActions>()
            .on_toggle_source(move |which| {
                if let Some(w) = weak.upgrade() {
                    let st = w.global::<LibraryAllState>();
                    match which.as_str() {
                        "purchases" => st.set_show_purchases(!st.get_show_purchases()),
                        "favorites" => st.set_show_favorites(!st.get_show_favorites()),
                        "following" => st.set_show_following(!st.get_show_following()),
                        "local" => st.set_show_local(!st.get_show_local()),
                        _ => {}
                    }
                    library_all::derive(&w);
                    let jobs = library_all::artwork_jobs(&w);
                    artwork::spawn_search_loads(jobs, weak.clone(), image_cache.clone());
                }
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window.global::<LibraryAllActions>().on_retry(move || {
            navigate_library_all(
                runtime.clone(),
                weak.clone(),
                &handle,
                image_cache.clone(),
            );
        });
    }
    {
        // Local search over the loaded favorite albums (title / artist).
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_albums_search(move |q| {
                if let Some(w) = weak.upgrade() {
                    w.global::<FavoritesState>().set_albums_search(q);
                    favorites::derive_albums(&w);
                }
            });
    }
    {
        // Sort the favorite albums (default / title / artist).
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_albums_set_sort(move |s| {
                if let Some(w) = weak.upgrade() {
                    w.global::<FavoritesState>().set_albums_sort_by(s);
                    favorites::derive_albums(&w);
                    favorites_prefs::save(&w);
                }
            });
    }
    {
        // Albums grid/list view toggle (persisted).
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_albums_set_view(move |v| {
                if let Some(w) = weak.upgrade() {
                    w.global::<FavoritesState>().set_albums_view_mode(v);
                    // Switching to the (non-windowed) list view needs covers
                    // the grid's window may have evicted.
                    favorites::albums_view_mode_changed(&w);
                    favorites_prefs::save(&w);
                }
            });
    }
    {
        // Windowed albums grid: dispatch covers for the reported row band
        // and evict the ones far outside it.
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_albums_window_changed(move |first, last| {
                if let Some(w) = weak.upgrade() {
                    favorites::albums_window_changed(&w, first, last);
                }
            });
    }
}
