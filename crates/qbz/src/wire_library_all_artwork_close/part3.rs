use crate::*;

pub(crate) fn wire_library_all_artwork_close_part3(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        // Retry loading the current favorites tab after a load error.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<FavoritesActions>()
            .on_retry_load(move || {
                if let Some(w) = weak.upgrade() {
                    let tab_id = w.global::<FavoritesState>().get_active_tab().to_string();
                    if let Some(tab) = favorites::FavTab::from_tab_id(&tab_id) {
                        navigate_favorites(
                            runtime.clone(),
                            weak.clone(),
                            &handle,
                            image_cache.clone(),
                            tab,
                            &tab_id,
                        );
                    }
                }
            });
    }
    {
        // Local search over the loaded favorite tracks (title / artist /
        // album), re-deriving the rendered list.
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_search_tracks(move |q| {
                if let Some(w) = weak.upgrade() {
                    w.global::<FavoritesState>().set_tracks_search(q);
                    favorites::derive_tracks(&w);
                }
            });
    }
    {
        // Local search over the loaded favorite labels (name).
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_search_labels(move |q| {
                if let Some(w) = weak.upgrade() {
                    w.global::<FavoritesState>().set_labels_search(q);
                    favorites::derive_labels(&w);
                }
            });
    }
    {
        // Group the favorite tracks (off / album / artist / name).
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_tracks_set_group(move |g| {
                if let Some(w) = weak.upgrade() {
                    w.global::<FavoritesState>().set_tracks_group_mode(g);
                    favorites::derive_tracks(&w);
                    favorites_prefs::save(&w);
                }
            });
    }
    {
        // Play all favorite tracks as a fresh queue.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<FavoritesActions>()
            .on_play_all_tracks(move || {
                playback::play_tracks(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    favorites::play_tracks(),
                    0,
                );
            });
    }
    {
        // Shuffle-play the favorite tracks.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<FavoritesActions>()
            .on_shuffle_tracks(move || {
                playback::play_tracks(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    favorites::shuffled_tracks(),
                    0,
                );
            });
    }
    {
        // Enter / leave the tracks multi-select edit mode.
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_toggle_multi_select(move || {
                if let Some(w) = weak.upgrade() {
                    let on = w.global::<FavoritesState>().get_tracks_multi_select();
                    favorites::set_multi_select(&w, !on);
                }
            });
    }
}
