use crate::*;

pub(crate) fn wire_library_all_artwork_close_part4(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        // Bulk bar actions over the selected favorite tracks.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<FavoritesActions>()
            .on_bulk_action(move |action| {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                match action.as_str() {
                    "select-all" => favorites::select_all(&w),
                    "clear" => favorites::clear_selection(&w),
                    "queue" => {
                        let tracks = favorites::selected_tracks(&w);
                        playback::enqueue_tracks(runtime.clone(), handle.clone(), tracks, false);
                    }
                    "play-next" => {
                        let tracks = favorites::selected_tracks(&w);
                        playback::enqueue_tracks(runtime.clone(), handle.clone(), tracks, true);
                    }
                    "make-offline" => {
                        let tracks = favorites::selected_tracks(&w);
                        offline_cache::cache_tracks(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            tracks,
                        );
                        favorites::clear_selection(&w);
                    }
                    "add-to-mixtape" => {
                        let items =
                            mixtape_items_from_qobuz_tracks(&favorites::selected_tracks(&w));
                        if !items.is_empty() {
                            open_add_to_mixtape(weak.clone(), handle.clone(), items);
                            favorites::clear_selection(&w);
                        }
                    }
                    "add-to-playlist" => {
                        let ids = favorites::selected_ids(&w);
                        if !ids.is_empty() {
                            playlist_picker::open_multi(&w, &ids, false);
                            let runtime = runtime.clone();
                            let weak = weak.clone();
                            handle.spawn(async move {
                                let playlists = playlist_picker::load(&runtime).await;
                                let _ = weak.upgrade_in_event_loop(move |w| {
                                    playlist_picker::apply(&w, playlists);
                                });
                            });
                        }
                    }
                    "remove-selected" => {
                        // Offline = read-only hearts (spec 4.3).
                        if offline_mode::engine().is_offline() {
                            toast::info(&w, "Not available offline");
                            return;
                        }
                        let ids = favorites::selected_ids(&w);
                        if ids.is_empty() {
                            return;
                        }
                        let runtime = runtime.clone();
                        let weak = weak.clone();
                        let handle = handle.clone();
                        let image_cache = image_cache.clone();
                        handle.clone().spawn(async move {
                            for id in &ids {
                                if let Err(e) =
                                    runtime.core().remove_favorite("track", id).await
                                {
                                    log::error!(
                                        "[qbz-slint] bulk remove favorite {id} failed: {e}"
                                    );
                                }
                                if let Ok(tid) = id.parse::<u64>() {
                                    crate::fav_cache::set(tid, false);
                                }
                            }
                            let _ = weak.upgrade_in_event_loop(|w| {
                                favorites::set_multi_select(&w, false);
                            });
                            navigate_favorites(
                                runtime.clone(),
                                weak.clone(),
                                &handle,
                                image_cache.clone(),
                                favorites::FavTab::Tracks,
                                "tracks",
                            );
                        });
                    }
                    _ => {}
                }
            });
    }
}
