use crate::*;

/// Open a card -> the collection-detail view (Phase-2 Slice 3). NAV-IN:
/// record history + navigate (loads via myqbz_detail::navigate), mirroring
/// the grid's own nav and the album/playlist detail openers.
pub(crate) fn wire_myqbz_open(
    window: &AppWindow,
    app_runtime: &Arc<AppRuntime<SlintAdapter>>,
    tokio_rt: &tokio::runtime::Runtime,
    image_cache: &artwork::ImageCache,
) {
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        let runtime = app_runtime.clone();
        window.global::<MyQbzActions>().on_open_card(move |id| {
            nav::record(nav::NavEntry::MixtapeDetail(id.to_string()));
            myqbz_detail::navigate(
                runtime.clone(),
                weak.clone(),
                handle.clone(),
                image_cache.clone(),
                id.to_string(),
            );
        });
    }
}
