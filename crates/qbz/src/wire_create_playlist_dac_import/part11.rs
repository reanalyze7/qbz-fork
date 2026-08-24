use crate::*;

pub(crate) fn wire_create_playlist_dac_import_part11(window: &AppWindow, _app_runtime: &Arc<AppRuntime<SlintAdapter>>, _tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, _settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_play_track(move |id| {
                if let Some(w) = weak.upgrade() {
                    w.invoke_media_action("track".into(), id, "play".into());
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_track_action(move |id, action| {
                if let Some(w) = weak.upgrade() {
                    w.invoke_media_action("track".into(), id, action);
                }
            });
    }
    {
        // Favorite album card actions (play / queue / favorite / go-to)
        // route through the album media-action arms.
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_album_action(move |id, action| {
                if let Some(w) = weak.upgrade() {
                    w.invoke_media_action("album".into(), id, action);
                }
            });
    }
    // ── Library "All" mixed feed — toolbar handlers ──
    {
        let weak = window.as_weak();
        let image_cache = image_cache.clone();
        window.global::<LibraryAllActions>().on_search(move |q| {
            if let Some(w) = weak.upgrade() {
                w.global::<LibraryAllState>().set_search(q);
                library_all::derive(&w);
                let jobs = library_all::artwork_jobs(&w);
                artwork::spawn_search_loads(jobs, weak.clone(), image_cache.clone());
            }
        });
    }
}
