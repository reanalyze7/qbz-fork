use crate::*;

pub(crate) fn wire_local_library_settings_part12(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        let weak = window.as_weak();
        window
            .global::<LocalLibraryActions>()
            .on_folders_set_group(move |group| {
                if let Some(w) = weak.upgrade() {
                    w.global::<LocalLibraryState>().set_folders_group(group);
                    local_library::derive_folders(&w);
                }
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalLibraryActions>()
            .on_folders_set_mode(move |mode| {
                if let Some(w) = weak.upgrade() {
                    w.global::<LocalLibraryState>()
                        .set_folders_view_mode(mode.clone());
                }
                // Lazy-load the tree roots the first time tree mode is shown.
                if mode.as_str() == "tree" {
                    local_library::ensure_folder_tree_loaded(weak.clone(), handle.clone());
                }
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalLibraryActions>()
            .on_folders_toggle_node(move |path, expand| {
                local_library::toggle_folder_node(
                    weak.clone(),
                    handle.clone(),
                    path.to_string(),
                    expand,
                );
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<LocalLibraryActions>()
            .on_folders_select(move |path, segment| {
                local_library::select_folder(
                    weak.clone(),
                    handle.clone(),
                    image_cache.clone(),
                    path.to_string(),
                    segment.to_string(),
                );
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<LocalLibraryActions>()
            .on_folder_detail_search(move |query| {
                if let Some(w) = weak.upgrade() {
                    local_library::folder_detail_search(&w, query.as_str());
                }
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalLibraryActions>()
            .on_folders_play_node(move |path| {
                playback::play_local_folder_recursive(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    path.to_string(),
                );
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalLibraryActions>()
            .on_folders_play_track(move |id| {
                if let Ok(row_id) = id.parse::<i64>() {
                    let path = weak
                        .upgrade()
                        .map(|w| {
                            w.global::<LocalLibraryState>()
                                .get_folders_selected_path()
                                .to_string()
                        })
                        .unwrap_or_default();
                    if !path.is_empty() {
                        playback::play_local_folder_tracks_from(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            path,
                            row_id,
                        );
                    }
                }
            });
    }
}
