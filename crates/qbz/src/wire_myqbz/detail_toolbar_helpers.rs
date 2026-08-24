use crate::*;

// After a toolbar re-derive the rendered model changed, so the visible rows
// need their thumbnails reloaded — through the SOURCE-SPLIT dispatch (Qobuz
// CDN urls via HTTP; local paths via the source-aware decoder).
pub(crate) fn refresh_row_covers(window: &AppWindow, image_cache: &artwork::ImageCache) {
    let split = myqbz_detail::artwork_jobs(window);
    myqbz_detail::dispatch_artwork(split, window.as_weak(), image_cache.clone());
}

// A toolbar re-derive rebuilds the rendered model with fresh rows
// (tracks_loaded reset to false). While in expanded view-mode the new
// visible rows must (re-)fetch their inline tracks (spec §8 auto-fetch).
pub(crate) fn ensure_expanded_if_active(
    window: &AppWindow,
    runtime: &Arc<AppRuntime<SlintAdapter>>,
    handle: &tokio::runtime::Handle,
) {
    if window.global::<MyQbzDetailState>().get_view_mode() == "expanded" {
        myqbz_detail::ensure_expanded(runtime.clone(), window.as_weak(), handle.clone());
    }
}
