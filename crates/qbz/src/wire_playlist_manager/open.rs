use crate::*;

// --- Open playlist -----------------------------------------------------
pub(crate) fn wire_pm_open(
    window: &AppWindow,
    app_runtime: &Arc<AppRuntime<SlintAdapter>>,
    tokio_rt: &tokio::runtime::Runtime,
    image_cache: &artwork::ImageCache,
) {
    let runtime = app_runtime.clone();
    let weak = window.as_weak();
    let handle = tokio_rt.handle().clone();
    let image_cache = image_cache.clone();
    window
        .global::<PlaylistManagerActions>()
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
