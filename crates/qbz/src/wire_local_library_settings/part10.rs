use crate::*;

pub(crate) fn wire_local_library_settings_part10(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        // Albums-grid bulk bar. Album->tracks resolution is a blocking DB read
        // (fetch_album_tracks_blocking), so it runs on spawn_blocking; the
        // resulting LocalTracks feed the same enqueue/playlist/mixtape paths as
        // the Tracks tab.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalLibraryActions>()
            .on_albums_bulk_action(move |action| {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                match action.as_str() {
                    "select-all" => local_library::select_all_albums(&w),
                    "clear" => local_library::clear_albums_selection(&w),
                    "queue" | "play-next" => {
                        let keys = local_library::selected_album_ids(&w);
                        let play_next = action.as_str() == "play-next";
                        let runtime = runtime.clone();
                        let handle2 = handle.clone();
                        handle.spawn(async move {
                            let rows = tokio::task::spawn_blocking(move || {
                                local_library::selected_albums_tracks_blocking(&keys)
                            })
                            .await
                            .unwrap_or_default();
                            playback::enqueue_local_tracks(runtime, handle2, rows, play_next);
                        });
                        local_library::clear_albums_selection(&w);
                    }
                    "add-to-playlist" => {
                        let keys = local_library::selected_album_ids(&w);
                        if !keys.is_empty() {
                            let runtime = runtime.clone();
                            let weak2 = weak.clone();
                            handle.spawn(async move {
                                let rows = tokio::task::spawn_blocking(move || {
                                    local_library::selected_albums_tracks_blocking(&keys)
                                })
                                .await
                                .unwrap_or_default();
                                let ids: Vec<String> = rows.iter().map(local_picker_ref).collect();
                                let runtime2 = runtime.clone();
                                let _ = weak2.upgrade_in_event_loop(move |w| {
                                    if !ids.is_empty() {
                                        playlist_picker::open_multi(&w, &ids, true);
                                    }
                                });
                                let playlists = playlist_picker::load(&runtime2).await;
                                let _ = weak2.upgrade_in_event_loop(move |w| {
                                    playlist_picker::apply(&w, playlists);
                                });
                            });
                        }
                    }
                    "add-to-mixtape" => {
                        let keys = local_library::selected_album_ids(&w);
                        if !keys.is_empty() {
                            let weak2 = weak.clone();
                            let handle2 = handle.clone();
                            handle.spawn(async move {
                                let rows = tokio::task::spawn_blocking(move || {
                                    local_library::selected_albums_tracks_blocking(&keys)
                                })
                                .await
                                .unwrap_or_default();
                                let _ = weak2.upgrade_in_event_loop(move |w| {
                                    let items = myqbz_add::track_items_from_local(&rows);
                                    if !items.is_empty() {
                                        open_add_to_mixtape(w.as_weak(), handle2, items);
                                        local_library::clear_albums_selection(&w);
                                    }
                                });
                            });
                        }
                    }
                    _ => {}
                }
            });
    }

    // ---- Folders tree rail: search / collapse / multi-select ----
    {
        let weak = window.as_weak();
        window
            .global::<LocalLibraryActions>()
            .on_folders_tree_search(move |query| {
                if let Some(w) = weak.upgrade() {
                    local_library::folders_tree_search(&w, query.as_str());
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<LocalLibraryActions>()
            .on_folders_collapse_all(move || {
                if let Some(w) = weak.upgrade() {
                    local_library::collapse_all_tree(&w);
                }
            });
    }
}
