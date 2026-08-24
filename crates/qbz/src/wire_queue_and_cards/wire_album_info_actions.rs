use crate::*;

/// Album Info modal actions (close / set-tab / play-track / open-label /
/// open-musician). Navigation reuses the same handlers the rest of the app
/// uses.
pub(crate) fn wire_album_info_actions(
    window: &AppWindow,
    app_runtime: &Arc<AppRuntime<SlintAdapter>>,
    tokio_rt: &tokio::runtime::Runtime,
    image_cache: &artwork::ImageCache,
) {
    let runtime = app_runtime.clone();
        // -- Album Info --
        let weak = window.as_weak();
        window
            .global::<AlbumInfoActions>()
            .on_close(move || {
                if let Some(w) = weak.upgrade() {
                    w.global::<AlbumInfoState>().set_open(false);
                }
            });
        let weak = window.as_weak();
        window
            .global::<AlbumInfoActions>()
            .on_set_tab(move |tab| {
                if let Some(w) = weak.upgrade() {
                    w.global::<AlbumInfoState>().set_active_tab(tab);
                }
            });
        let weak = window.as_weak();
        let runtime_p = runtime.clone();
        let handle_p = tokio_rt.handle().clone();
        window
            .global::<AlbumInfoActions>()
            .on_play_track(move |id| {
                if let Some(w) = weak.upgrade() {
                    // Album view is the modal's context, so this plays the
                    // album starting at the chosen track (Tauri keeps the
                    // modal open on play).
                    playback::play_track_in_context(
                        &w,
                        runtime_p.clone(),
                        w.as_weak(),
                        handle_p.clone(),
                        &id,
                    );
                }
            });
        let weak = window.as_weak();
        let runtime_a = runtime.clone();
        let handle_a = tokio_rt.handle().clone();
        let image_cache_a = image_cache.clone();
        window
            .global::<AlbumInfoActions>()
            .on_open_label(move |label_id| {
                if let Some(w) = weak.upgrade() {
                    let name = w.global::<AlbumInfoState>().get_label().to_string();
                    w.global::<AlbumInfoState>().set_open(false);
                    if let Ok(id) = label_id.parse::<u64>() {
                        navigate_label(
                            runtime_a.clone(),
                            w.as_weak(),
                            &handle_a,
                            image_cache_a.clone(),
                            id,
                            name,
                        );
                    }
                }
            });
        let weak = window.as_weak();
        window
            .global::<AlbumInfoActions>()
            .on_open_musician(move |name, role| {
                if let Some(w) = weak.upgrade() {
                    w.global::<AlbumInfoState>().set_open(false);
                    w.global::<NetworkSidebarActions>()
                        .invoke_musician_clicked(name, role);
                }
            });
}
