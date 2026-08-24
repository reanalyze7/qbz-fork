use crate::*;

pub(crate) fn wire_playlist_browse_picker_part2(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, _settings_ctx: &Arc<settings::SettingsCtx>) {

    // Discover "View all" search — re-filter the loaded albums
    // client-side after the search box changes (UI thread).
    {
        let weak = window.as_weak();
        window
            .global::<DiscoverBrowseActions>()
            .on_search_changed(move || {
                if let Some(w) = weak.upgrade() {
                    discover_browse::apply_filter(&w);
                }
            });
    }

    // Qobuz Playlists "View all" — pagination, client-side search and the
    // single-select category tag bar (server-side re-fetch from offset 0).
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<PlaylistBrowseActions>()
            .on_load_more(move || {
                playlist_browse::load_more(
                    runtime.clone(),
                    weak.clone(),
                    &handle,
                    image_cache.clone(),
                    current_genre_filter(),
                );
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<PlaylistBrowseActions>()
            .on_search_changed(move || {
                if let Some(w) = weak.upgrade() {
                    playlist_browse::apply_filter(&w);
                }
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<PlaylistBrowseActions>()
            .on_select_tag(move |slug| {
                playlist_browse::select_tag(
                    runtime.clone(),
                    weak.clone(),
                    &handle,
                    image_cache.clone(),
                    slug.to_string(),
                    current_genre_filter(),
                );
            });
    }
}
