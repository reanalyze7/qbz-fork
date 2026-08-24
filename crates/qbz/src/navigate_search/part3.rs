use crate::*;

/// Open an ArtistsByLocationView for the given scene params. Runs the
/// discovery on a worker, applies the validated artist grid, then
/// fans out artwork jobs for the candidates' Qobuz thumbnails.
pub(crate) fn navigate_location(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: artwork::ImageCache,
    params: artist::LocationParams,
) {
    handle.spawn(async move {
        let _ = weak.upgrade_in_event_loop(|w| {
            location_view::reset_scene(&w);
            w.global::<NavState>().set_view(ContentView::Location);
        });
        match location_view::load_scene(&runtime, &params, 0).await {
            Ok(data) => {
                let jobs = location_view::artwork_jobs(&data);
                let _ = weak.upgrade_in_event_loop(move |w| {
                    location_view::apply_scene(&w, data);
                });
                artwork::spawn_loads(jobs, weak.clone(), image_cache.clone());
            }
            Err(e) => {
                log::error!("[qbz-slint] scene discovery failed: {e}");
                let _ = weak.upgrade_in_event_loop(|w| {
                    w.global::<LocationViewState>().set_loading(false);
                });
            }
        }
    });
}

