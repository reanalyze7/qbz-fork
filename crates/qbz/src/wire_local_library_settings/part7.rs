use crate::*;

pub(crate) fn wire_local_library_settings_part7(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<LocalLibraryActions>()
            .on_open_album(move |id| {
                nav::record(nav::NavEntry::LocalAlbum(id.to_string()));
                navigate_local_album(
                    runtime.clone(),
                    weak.clone(),
                    &handle,
                    image_cache.clone(),
                    id.to_string(),
                );
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<LocalLibraryActions>()
            .on_open_artist(move |name| {
                // `name` is the artist NAME (local artists have no id).
                open_local_artist(&runtime, &weak, &handle, &image_cache, name.to_string());
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalLibraryActions>()
            .on_album_action(move |id, action| match action.as_str() {
                "play" => {
                    // The whole album becomes the queue and auto-advances.
                    playback::play_local_album(
                        runtime.clone(),
                        weak.clone(),
                        handle.clone(),
                        id.to_string(),
                        None,
                    );
                }
                "toggle-select" => {
                    if let Some(w) = weak.upgrade() {
                        local_library::toggle_album_select(&w, id.as_str());
                    }
                }
                "favorite" => {
                    if let Some(w) = weak.upgrade() {
                        local_library::toggle_album_favorite(&w, id.as_str());
                    }
                }
                "play-next" | "queue" => {
                    // Single-album play-next / queue (#636 — this arm used to
                    // be a "queue slice pending" stub): resolve the album's
                    // tracks source-aware (local folders, the same
                    // resolver `play` uses) and enqueue the whole album
                    // without starting playback.
                    let play_next = action.as_str() == "play-next";
                    let runtime = runtime.clone();
                    let handle2 = handle.clone();
                    let album_id = id.to_string();
                    handle.spawn(async move {
                        let rows = tokio::task::spawn_blocking(move || {
                            local_library::fetch_album_tracks_blocking(&album_id)
                        })
                        .await
                        .unwrap_or_default();
                        playback::enqueue_local_tracks(runtime, handle2, rows, play_next);
                    });
                }
                _ => {
                    log::debug!("[qbz-slint] unhandled local album action: {id} {action}");
                }
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalLibraryActions>()
            .on_tracks_search(move |_query| {
                // The query is two-way bound to tracks-search; reload page 1.
                local_library::reload_tracks(weak.clone(), handle.clone());
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalLibraryActions>()
            .on_tracks_load_more(move || {
                local_library::load_more_tracks(weak.clone(), handle.clone());
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalLibraryActions>()
            .on_tracks_retry(move || {
                local_library::reload_tracks(weak.clone(), handle.clone());
            });
    }
}
