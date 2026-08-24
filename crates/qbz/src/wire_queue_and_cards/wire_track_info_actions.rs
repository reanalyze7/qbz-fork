use crate::*;

/// Track Info modal actions (close / open-artist / open-label /
/// open-musician / load-inline for the Immersive split panel). Navigation
/// reuses the same handlers the rest of the app uses.
pub(crate) fn wire_track_info_actions(
    window: &AppWindow,
    app_runtime: &Arc<AppRuntime<SlintAdapter>>,
    tokio_rt: &tokio::runtime::Runtime,
    image_cache: &artwork::ImageCache,
) {
    let _ = image_cache;
        let runtime = app_runtime.clone();
        // -- Track Info --
        let weak = window.as_weak();
        window
            .global::<TrackInfoActions>()
            .on_close(move || {
                if let Some(w) = weak.upgrade() {
                    w.global::<TrackInfoState>().set_open(false);
                }
            });
        let weak = window.as_weak();
        window
            .global::<TrackInfoActions>()
            .on_open_artist(move |artist_id| {
                if let Some(w) = weak.upgrade() {
                    w.global::<TrackInfoState>().set_open(false);
                    w.invoke_open_artist(artist_id);
                }
            });
        let weak = window.as_weak();
        let runtime_l = runtime.clone();
        let handle_l = tokio_rt.handle().clone();
        let image_cache_l = image_cache.clone();
        window
            .global::<TrackInfoActions>()
            .on_open_label(move |label_id| {
                if let Some(w) = weak.upgrade() {
                    let name = w.global::<TrackInfoState>().get_label().to_string();
                    w.global::<TrackInfoState>().set_open(false);
                    if let Ok(id) = label_id.parse::<u64>() {
                        navigate_label(
                            runtime_l.clone(),
                            w.as_weak(),
                            &handle_l,
                            image_cache_l.clone(),
                            id,
                            name,
                        );
                    }
                }
            });
        let weak = window.as_weak();
        window
            .global::<TrackInfoActions>()
            .on_open_musician(move |name, role| {
                if let Some(w) = weak.upgrade() {
                    w.global::<TrackInfoState>().set_open(false);
                    w.global::<NetworkSidebarActions>()
                        .invoke_musician_clicked(name, role);
                }
            });
        // Immersive split Track Info panel: populate TrackInfoState for the
        // given track WITHOUT opening the floating modal (open stays false).
        let weak = window.as_weak();
        let runtime_l = runtime.clone();
        let handle_l = tokio_rt.handle().clone();
        window
            .global::<TrackInfoActions>()
            .on_load_inline(move |track_id| {
                if let Ok(id) = track_id.parse::<u64>() {
                    info_modals::load_track_info_inline(
                        runtime_l.clone(),
                        weak.clone(),
                        handle_l.clone(),
                        id,
                    );
                }
            });
}
