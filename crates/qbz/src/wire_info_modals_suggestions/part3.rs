use crate::*;

pub(crate) fn wire_info_modals_suggestions_part3(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<SuggestionsActions>()
            .on_queue_playlist(move |playlist_id| {
                let id = playlist_id.to_string();
                if id.is_empty() {
                    return;
                }
                playback::enqueue_playlist(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    id,
                    false,
                );
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<SuggestionsActions>()
            .on_play_next_playlist(move |playlist_id| {
                let id = playlist_id.to_string();
                if id.is_empty() {
                    return;
                }
                playback::enqueue_playlist(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    id,
                    true,
                );
            });
    }
    {
        // play-track — play a single recommended track by id NOW.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<SuggestionsActions>()
            .on_play_track(move |track_id| {
                let Ok(tid) = track_id.parse::<u64>() else {
                    return;
                };
                let runtime = runtime.clone();
                let weak = weak.clone();
                let handle2 = handle.clone();
                handle.spawn(async move {
                    match runtime.core().get_track(tid).await {
                        Ok(track) => {
                            playback::play_tracks(runtime, weak, handle2, vec![track], 0);
                        }
                        Err(e) => {
                            log::error!("[qbz-slint] suggestions play-track {tid} failed: {e}");
                        }
                    }
                });
            });
    }

    // --- Playlist "Suggested Songs" section (T8) ----------------------------
    // 1:1 port of the Svelte PlaylistSuggestions component. The pool +
    // pagination + dedupe live in crate::playlist_suggestions; the nav actions
    // route through the shared media-action arms the playlist track rows use.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<PlaylistSuggestionsActions>()
            .on_activate(move || {
                if let Some(w) = weak.upgrade() {
                    playlist_suggestions::activate(&w, runtime.clone(), handle.clone());
                }
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<PlaylistSuggestionsActions>()
            .on_refresh(move || {
                if let Some(w) = weak.upgrade() {
                    playlist_suggestions::refresh(&w, runtime.clone(), handle.clone());
                }
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<PlaylistSuggestionsActions>()
            .on_add_track(move |track_id| {
                if let Some(w) = weak.upgrade() {
                    playlist_suggestions::add_track(
                        &w,
                        runtime.clone(),
                        handle.clone(),
                        track_id.to_string(),
                    );
                }
            });
    }
}
