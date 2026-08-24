use crate::*;

pub(crate) fn wire_discover_offline_manager_part5(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, _settings_ctx: &Arc<settings::SettingsCtx>) {

    // Home "Recently Played Albums" rail "View all" -> the full page listing
    // the local play-history albums (record history, navigate, refresh the
    // nav flags).
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<HomeActions>()
            .on_open_recent_albums(move || {
                nav::record(nav::NavEntry::RecentAlbums);
                navigate_recent_albums(weak.clone(), &handle, image_cache.clone());
                if let Some(w) = weak.upgrade() {
                    update_nav_flags(&w);
                }
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<HomeActions>()
            .on_open_most_played_albums(move || {
                nav::record(nav::NavEntry::MostPlayedAlbums);
                navigate_most_played_albums(weak.clone(), &handle, image_cache.clone());
                if let Some(w) = weak.upgrade() {
                    update_nav_flags(&w);
                }
            });
    }
    {
        let weak = window.as_weak();
        let image_cache = image_cache.clone();
        window
            .global::<MostPlayedAlbumsActions>()
            .on_filter(move |q| {
                filter_most_played(weak.clone(), image_cache.clone(), q.to_string());
            });
    }

    // Qobuz Playlists rail "View all" -> the full-page playlist browse
    // (server-side tag + genre filtering). A fresh open resets the
    // category tab to All (Tauri parity); genre-filter and history
    // re-navigations preserve it.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<HomeActions>()
            .on_open_playlist_browse(move || {
                nav::record(nav::NavEntry::PlaylistBrowse);
                if let Some(w) = weak.upgrade() {
                    update_nav_flags(&w);
                }
                playlist_browse::navigate(
                    runtime.clone(),
                    weak.clone(),
                    &handle,
                    image_cache.clone(),
                    current_genre_filter(),
                    true,
                );
            });
    }

    // Recently-played rails refresh. `home-mounted` fires on every HomeView
    // (re)mount: re-read the LOCAL store into the rails IF a play was recorded
    // while Home was off-screen (dirty flag — a no-op otherwise, so mounting
    // Home stays free). While Home IS showing, playback refreshes the rails
    // directly (note_recent_store_changed). No polling anywhere.
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window.global::<HomeActions>().on_home_mounted(move || {
            if RECENT_RAILS_DIRTY.load(std::sync::atomic::Ordering::Relaxed) {
                refresh_recent_rails(weak.clone(), &handle, image_cache.clone());
            }
        });
    }
    // Manual refresh (the toolbar button next to the nav cluster): an
    // unconditional local re-read of the recently-played rails on demand.
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window.global::<HomeActions>().on_refresh_recent(move || {
            refresh_recent_rails(weak.clone(), &handle, image_cache.clone());
        });
    }
    // Library Albums (#566) header sort: reorder the cached favorite-albums
    // base list (0 recent / 1 first / 2 random) and re-push it + its covers.
    {
        let weak = window.as_weak();
        let image_cache = image_cache.clone();
        window.global::<HomeActions>().on_set_library_albums_sort(move |mode| {
            apply_library_albums_sort(weak.clone(), mode, image_cache.clone());
        });
    }
}
