use crate::*;

pub(crate) fn wire_discover_offline_manager_part9(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<GenreFilterActions>()
            .on_toggle_expand(move |id| {
                let now_expanded = genre_filter::toggle_expand(id.as_str());
                let Some(w) = weak.upgrade() else {
                    return;
                };
                genre_filter::apply_state(&w);
                // Lazy-load the node's children the first time it expands.
                if now_expanded {
                    if let Ok(gid) = id.to_string().parse::<u64>() {
                        if !genre_filter::children_loaded(gid) {
                            let runtime = runtime.clone();
                            let weak = weak.clone();
                            handle.spawn(async move {
                                genre_filter::load_children(&runtime, gid).await;
                                let _ = weak.upgrade_in_event_loop(|w| {
                                    genre_filter::apply_state(&w);
                                });
                            });
                        }
                    }
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<GenreFilterActions>()
            .on_search(move |query| {
                genre_filter::set_search(query.as_str());
                if let Some(w) = weak.upgrade() {
                    genre_filter::apply_state(&w);
                }
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<GenreFilterActions>()
            .on_clear(move || {
                genre_filter::clear();
                let Some(w) = weak.upgrade() else {
                    return;
                };
                genre_filter::apply_state(&w);
                if genre_filter::current_context() == "library-all" {
                    library_all::derive(&w);
                    let jobs = library_all::artwork_jobs(&w);
                    artwork::spawn_search_loads(jobs, w.as_weak(), image_cache.clone());
                    return;
                }
                if genre_filter::current_context() == "favorites" {
                    if w.global::<FavoritesState>().get_active_tab().as_str() == "albums" {
                        favorites::derive_albums(&w);
                    } else {
                        favorites::derive_tracks(&w);
                    }
                    return;
                }
                let browse_target = current_browse_target(&w);
                let playlist_browse_showing = current_playlist_browse_showing(&w);
                if browse_target.is_none() && !playlist_browse_showing {
                    w.global::<HomeState>().set_loading(true);
                }
                let active = w.global::<HomeState>().get_active_tab().to_string();
                let runtime = runtime.clone();
                let weak = weak.clone();
                let image_cache = image_cache.clone();
                let handle2 = handle.clone();
                handle.spawn(async move {
                    if let Some((endpoint, title)) = browse_target {
                        discover_browse::navigate(
                            runtime.clone(),
                            weak.clone(),
                            &handle2,
                            image_cache.clone(),
                            endpoint,
                            title,
                            current_genre_filter(),
                        );
                    } else if playlist_browse_showing {
                        // Re-navigation preserves the page's selected tag
                        // (reset_tag = false).
                        playlist_browse::navigate(
                            runtime.clone(),
                            weak.clone(),
                            &handle2,
                            image_cache.clone(),
                            current_genre_filter(),
                            false,
                        );
                    } else {
                        reload_home(&runtime, &weak, &image_cache, active).await;
                    }
                });
            });
    }
}
