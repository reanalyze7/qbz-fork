use crate::*;

pub(crate) fn wire_discover_offline_manager_part8(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, _settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<GenreFilterActions>()
            .on_toggle(move |id| {
                let was_selected = genre_filter::selected_ids()
                    .iter()
                    .any(|x| x.to_string() == id.as_str());
                if !genre_filter::toggle(id.as_str()) {
                    return;
                }
                let Some(w) = weak.upgrade() else {
                    return;
                };
                genre_filter::apply_state(&w);
                // Library "All": client-side genre filter over the mixed feed.
                if genre_filter::current_context() == "library-all" {
                    let runtime_f = runtime.clone();
                    let weak_f = weak.clone();
                    let image_cache_f = image_cache.clone();
                    let id_f = id.to_string();
                    handle.spawn(async move {
                        if !was_selected {
                            if let Ok(gid) = id_f.parse::<u64>() {
                                genre_filter::load_descendants(&runtime_f, gid).await;
                            }
                        }
                        let _ = weak_f.upgrade_in_event_loop(move |w| {
                            genre_filter::apply_state(&w);
                            library_all::derive(&w);
                            let jobs = library_all::artwork_jobs(&w);
                            artwork::spawn_search_loads(jobs, w.as_weak(), image_cache_f.clone());
                        });
                    });
                    return;
                }
                // Favorites: client-side genre filter — re-derive the active
                // favorites tab instead of re-fetching the discover index.
                if genre_filter::current_context() == "favorites" {
                    let runtime_f = runtime.clone();
                    let weak_f = weak.clone();
                    let id_f = id.to_string();
                    handle.spawn(async move {
                        if !was_selected {
                            if let Ok(gid) = id_f.parse::<u64>() {
                                genre_filter::load_descendants(&runtime_f, gid).await;
                            }
                        }
                        let _ = weak_f.upgrade_in_event_loop(|w| {
                            genre_filter::apply_state(&w);
                            if w.global::<FavoritesState>().get_active_tab().as_str() == "albums" {
                                favorites::derive_albums(&w);
                            } else {
                                favorites::derive_tracks(&w);
                            }
                        });
                    });
                    return;
                }
                // When a "View all" browse page is showing (albums OR the
                // Qobuz Playlists page), the genre change re-fetches THAT
                // page; otherwise it reloads the Discover home index.
                let browse_target = current_browse_target(&w);
                let playlist_browse_showing = current_playlist_browse_showing(&w);
                if browse_target.is_none() && !playlist_browse_showing {
                    w.global::<HomeState>().set_loading(true);
                }
                let active = w.global::<HomeState>().get_active_tab().to_string();
                let id = id.to_string();
                let runtime = runtime.clone();
                let weak = weak.clone();
                let image_cache = image_cache.clone();
                let handle2 = handle.clone();
                handle.spawn(async move {
                    // On a newly-selected genre, eager-load its descendants
                    // so selected_names covers the child genres (favorites)
                    // and the tree shows counts.
                    if !was_selected {
                        if let Ok(gid) = id.parse::<u64>() {
                            genre_filter::load_descendants(&runtime, gid).await;
                            let _ = weak.upgrade_in_event_loop(|w| {
                                genre_filter::apply_state(&w);
                            });
                        }
                    }
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
