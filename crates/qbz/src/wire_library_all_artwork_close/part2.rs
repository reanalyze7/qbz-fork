use crate::*;

pub(crate) fn wire_library_all_artwork_close_part2(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        // Group the favorite albums (off / alpha / artist).
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_albums_set_group(move |g| {
                if let Some(w) = weak.upgrade() {
                    w.global::<FavoritesState>().set_albums_group_mode(g);
                    favorites::derive_albums(&w);
                    favorites_prefs::save(&w);
                }
            });
    }
    {
        // Play a random album from the visible favorites set.
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_albums_shuffle(move || {
                if let Some(w) = weak.upgrade() {
                    if let Some(id) = favorites::random_visible_album(&w) {
                        w.invoke_media_action("album".into(), id.into(), "play".into());
                    }
                }
            });
    }
    {
        // Un-favorite a track from the favorites list: fade the row, remove
        // the favorite on the server, then drop the row after the fade.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<FavoritesActions>()
            .on_unfavorite_track(move |id| {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                // Offline = read-only hearts (spec 4.3).
                if offline_mode::engine().is_offline() {
                    toast::info(&w, "Not available offline");
                    return;
                }
                favorites::mark_track_removing(&w, &id);
                if let Ok(tid) = id.parse::<u64>() {
                    crate::fav_cache::set(tid, false);
                }
                let id_srv = id.to_string();
                let runtime = runtime.clone();
                handle.spawn(async move {
                    if let Err(e) = runtime.core().remove_favorite("track", &id_srv).await {
                        log::error!("[qbz-slint] unfavorite track {id_srv} failed: {e}");
                    }
                });
                let weak2 = weak.clone();
                let id_rm = id.to_string();
                slint::Timer::single_shot(std::time::Duration::from_millis(280), move || {
                    if let Some(w) = weak2.upgrade() {
                        favorites::remove_track_row(&w, &id_rm);
                    }
                });
            });
    }
    {
        // Un-favorite an album from the favorites list (same fade + remove).
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<FavoritesActions>()
            .on_unfavorite_album(move |id| {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                favorites::mark_album_removing(&w, &id);
                // Keep the favorite-album cache in sync so the album-header
                // heart reflects an unfavorite done from the Favorites view.
                crate::fav_cache::set_album(&id, false);
                // Empty the heart on any other surface currently showing this
                // album (artist discography, carousels, search) — the
                // favorites rows themselves fade out and are removed below.
                set_album_row_favorite(&w, &id, false);
                let id_srv = id.to_string();
                let runtime = runtime.clone();
                handle.spawn(async move {
                    if let Err(e) = runtime.core().remove_favorite("album", &id_srv).await {
                        log::error!("[qbz-slint] unfavorite album {id_srv} failed: {e}");
                    }
                });
                let weak2 = weak.clone();
                let id_rm = id.to_string();
                slint::Timer::single_shot(std::time::Duration::from_millis(280), move || {
                    if let Some(w) = weak2.upgrade() {
                        favorites::remove_album_row(&w, &id_rm);
                    }
                });
            });
    }
}
