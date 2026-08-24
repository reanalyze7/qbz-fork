use crate::*;

pub(crate) fn wire_discover_offline_manager_part2(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, _settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        let weak = window.as_weak();
        window
            .global::<BlacklistActions>()
            .on_remove_album(move |id| {
                if let Some(w) = weak.upgrade() {
                    blacklist_manager::remove_album(&w, id.to_string());
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<BlacklistActions>()
            .on_clear_all_albums(move || {
                if let Some(w) = weak.upgrade() {
                    blacklist_manager::clear_all_albums(&w);
                }
            });
    }
    // --- Reco-dismissal callbacks (the Recommendations tab) ---
    {
        let weak = window.as_weak();
        window
            .global::<BlacklistActions>()
            .on_remove_dismissed(move |id| {
                if let Some(w) = weak.upgrade() {
                    blacklist_manager::remove_dismissed(&w, id);
                }
            });
    }
    {
        let weak = window.as_weak();
        let bl_runtime_b = app_runtime.clone();
        let bl_handle_b = tokio_rt.handle().clone();
        let bl_image_cache_b = image_cache.clone();
        window
            .global::<BlacklistActions>()
            .on_album_select(move |id| {
                let album_id = id.to_string();
                nav::record(nav::NavEntry::Album(album_id.clone()));
                navigate_album(
                    bl_runtime_b.clone(),
                    weak.clone(),
                    &bl_handle_b,
                    bl_image_cache_b.clone(),
                    album_id,
                );
                if let Some(w) = weak.upgrade() {
                    update_nav_flags(&w);
                }
            });
    }
}
