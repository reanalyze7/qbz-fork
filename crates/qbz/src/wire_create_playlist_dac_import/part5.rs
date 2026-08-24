use crate::*;

pub(crate) fn wire_create_playlist_dac_import_part5(window: &AppWindow, _app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, _image_cache: &artwork::ImageCache, _settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        // Step A: fetch the preview (no session needed).
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<PlaylistImportActions>().on_fetch(move || {
            let Some(w) = weak.upgrade() else { return; };
            let Some(url) = playlist_import::begin_fetch(&w) else {
                return;
            };
            // A reopen mid-fetch bumps the generation; the stale preview
            // result must not land on the fresh modal (§1.8).
            let generation = playlist_import::current_generation();
            let weak = weak.clone();
            handle.spawn(async move {
                let res = qbz_playlist_import::preview_public_playlist(&url).await;
                let _ = weak.upgrade_in_event_loop(move |w| {
                    if generation != playlist_import::current_generation() {
                        return;
                    }
                    match res {
                        Ok(p) => playlist_import::apply_preview_ok(&w, &url, p),
                        Err(e) => playlist_import::apply_preview_err(&w, &e.to_string()),
                    }
                });
            });
        });
    }
}
