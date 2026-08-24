use crate::*;

pub(crate) fn wire_local_library_settings_part11(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        let weak = window.as_weak();
        window
            .global::<LocalLibraryActions>()
            .on_folders_toggle_select_mode(move || {
                if let Some(w) = weak.upgrade() {
                    local_library::toggle_tree_select_mode(&w);
                }
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalLibraryActions>()
            .on_folders_toggle_folder_select(move |path| {
                local_library::toggle_tree_folder_select(weak.clone(), handle.clone(), path.to_string());
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalLibraryActions>()
            .on_folders_toggle_track_select(move |path| {
                local_library::toggle_tree_track_select(weak.clone(), handle.clone(), path.to_string());
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalLibraryActions>()
            .on_folders_bulk_action(move |action| {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                match action.as_str() {
                    "select-all" => {
                        local_library::tree_select_all(weak.clone(), handle.clone());
                    }
                    "clear" => local_library::tree_clear_selection(&w),
                    "queue" => {
                        let rows = local_library::tree_selected_snapshot();
                        playback::enqueue_local_tracks(runtime.clone(), handle.clone(), rows, false);
                        local_library::tree_clear_selection(&w);
                    }
                    "play-next" => {
                        let rows = local_library::tree_selected_snapshot();
                        playback::enqueue_local_tracks(runtime.clone(), handle.clone(), rows, true);
                        local_library::tree_clear_selection(&w);
                    }
                    "add-to-playlist" => {
                        // Source-aware refs (library row ids).
                        let rows = local_library::tree_selected_snapshot();
                        let ids: Vec<String> = rows.iter().map(local_picker_ref).collect();
                        if !ids.is_empty() {
                            playlist_picker::open_multi(&w, &ids, true);
                            let runtime = runtime.clone();
                            let weak2 = weak.clone();
                            handle.spawn(async move {
                                let playlists = playlist_picker::load(&runtime).await;
                                let _ = weak2.upgrade_in_event_loop(move |w| {
                                    playlist_picker::apply(&w, playlists);
                                });
                            });
                        }
                    }
                    "add-to-mixtape" => {
                        // All selected tracks.
                        let rows = local_library::tree_selected_snapshot();
                        let items = myqbz_add::track_items_from_local(&rows);
                        if !items.is_empty() {
                            open_add_to_mixtape(weak.clone(), handle.clone(), items);
                            local_library::tree_clear_selection(&w);
                        }
                    }
                    _ => {}
                }
            });
    }

    // ---- Folders tab actions ----
    {
        let weak = window.as_weak();
        window
            .global::<LocalLibraryActions>()
            .on_folders_search(move |_query| {
                // Query is two-way bound to folders-search; re-derive in place.
                if let Some(w) = weak.upgrade() {
                    local_library::derive_folders(&w);
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<LocalLibraryActions>()
            .on_folders_set_sort(move |sort| {
                if let Some(w) = weak.upgrade() {
                    w.global::<LocalLibraryState>().set_folders_sort(sort);
                    local_library::derive_folders(&w);
                }
            });
    }
}
