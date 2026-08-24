use crate::*;

pub(crate) fn wire_local_library_settings_part9(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, _image_cache: &artwork::ImageCache, _settings_ctx: &Arc<settings::SettingsCtx>) {

    // ---- Tracks tab: sort + group-by + multi-select + bulk ----
    {
        let weak = window.as_weak();
        window
            .global::<LocalLibraryActions>()
            .on_tracks_set_group(move |mode| {
                if let Some(w) = weak.upgrade() {
                    local_library::set_tracks_group(&w, mode.as_str());
                }
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalLibraryActions>()
            .on_tracks_set_sort(move |key| {
                if let Some(w) = weak.upgrade() {
                    local_library::set_tracks_sort(&w, key.as_str(), handle.clone());
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<LocalLibraryActions>()
            .on_tracks_toggle_multi_select(move || {
                if let Some(w) = weak.upgrade() {
                    let on = w.global::<LocalLibraryState>().get_tracks_multi_select();
                    local_library::set_tracks_multi_select(&w, !on);
                }
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalLibraryActions>()
            .on_tracks_bulk_action(move |action| {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                match action.as_str() {
                    "select-all" => local_library::select_all_tracks(&w),
                    "clear" => local_library::clear_tracks_selection(&w),
                    "queue" => {
                        let rows = local_library::selected_local_tracks(&w);
                        playback::enqueue_local_tracks(runtime.clone(), handle.clone(), rows, false);
                        local_library::clear_tracks_selection(&w);
                    }
                    "play-next" => {
                        let rows = local_library::selected_local_tracks(&w);
                        playback::enqueue_local_tracks(runtime.clone(), handle.clone(), rows, true);
                        local_library::clear_tracks_selection(&w);
                    }
                    "add-to-playlist" => {
                        // Source-aware refs: library row ids (resolved at insert).
                        let rows = local_library::selected_local_tracks(&w);
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
                        let rows = local_library::selected_local_tracks(&w);
                        let items = myqbz_add::track_items_from_local(&rows);
                        if !items.is_empty() {
                            open_add_to_mixtape(weak.clone(), handle.clone(), items);
                            local_library::clear_tracks_selection(&w);
                        }
                    }
                    _ => {}
                }
            });
    }
    {
        // Albums-grid multi-select toggle.
        let weak = window.as_weak();
        window
            .global::<LocalLibraryActions>()
            .on_albums_toggle_multi_select(move || {
                if let Some(w) = weak.upgrade() {
                    let on = w.global::<LocalLibraryState>().get_albums_multi_select();
                    local_library::set_albums_multi_select(&w, !on);
                }
            });
    }
}
