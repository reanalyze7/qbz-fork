use crate::*;

pub(crate) fn wire_discover_offline_manager_part10(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, _image_cache: &artwork::ImageCache, _settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        let weak = window.as_weak();
        window
            .global::<GenreFilterActions>()
            .on_set_remember(move |v| {
                genre_filter::set_remember(v);
                if let Some(w) = weak.upgrade() {
                    genre_filter::apply_state(&w);
                }
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<GenreFilterActions>()
            .on_set_advanced(move |v| {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                w.global::<GenreFilterState>().set_advanced(v);
                // First time advanced view opens, eager-load every
                // parent's children so the tree shows child counts.
                if v {
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    handle.spawn(async move {
                        genre_filter::load_all_parent_children(&runtime).await;
                        let _ = weak.upgrade_in_event_loop(|w| {
                            genre_filter::apply_state(&w);
                        });
                    });
                }
            });
    }
}
