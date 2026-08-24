use crate::*;

pub(crate) fn wire_create_playlist_dac_import_part8(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, _settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        // Favorite playlist click — open the playlist detail view.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<FavoritesActions>()
            .on_open_playlist(move |id| {
                nav::record(nav::NavEntry::Playlist(id.to_string()));
                navigate_playlist(
                    runtime.clone(),
                    weak.clone(),
                    &handle,
                    image_cache.clone(),
                    id.to_string(),
                );
                if let Some(w) = weak.upgrade() {
                    update_nav_flags(&w);
                }
            });
    }
    {
        // Switch the Playlists sub-tab (Library / Following) + re-derive.
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_playlists_set_sub_tab(move |sub| {
                if let Some(w) = weak.upgrade() {
                    w.global::<FavoritesState>().set_playlists_sub_tab(sub);
                    favorites::derive_playlists(&w);
                }
            });
    }
    {
        // Local search over the loaded favorite playlists (name | owner).
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_search_playlists(move |q| {
                if let Some(w) = weak.upgrade() {
                    w.global::<FavoritesState>().set_playlists_search(q);
                    favorites::derive_playlists(&w);
                }
            });
    }
    {
        // Playlists grid/list view toggle (persisted).
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_playlists_set_view(move |v| {
                if let Some(w) = weak.upgrade() {
                    w.global::<FavoritesState>().set_playlists_view_mode(v);
                    favorites_prefs::save(&w);
                }
            });
    }
    {
        // Local search over the loaded favorite artists (name).
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_search_artists(move |q| {
                if let Some(w) = weak.upgrade() {
                    w.global::<FavoritesState>().set_artists_search(q);
                    favorites::derive_artists(&w);
                }
            });
    }
    {
        // Artists header Shuffle = open a random visible artist (random
        // ARTIST, not a random album — matches Tauri).
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_artists_shuffle(move || {
                if let Some(w) = weak.upgrade() {
                    if let Some(id) = favorites::random_visible_artist(&w) {
                        w.invoke_open_artist(id.into());
                    }
                }
            });
    }
    {
        // Playlists "random" — play a random visible playlist (reuses the
        // playlist-action "play" path).
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_playlists_random(move || {
                if let Some(w) = weak.upgrade() {
                    if let Some(id) = favorites::random_visible_playlist(&w) {
                        w.global::<FavoritesActions>()
                            .invoke_playlist_action(id.into(), "play".into());
                    }
                }
            });
    }
}
