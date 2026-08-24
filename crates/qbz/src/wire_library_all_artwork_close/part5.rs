use crate::*;

pub(crate) fn wire_library_all_artwork_close_part5(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {

    // Artwork right-click menu wiring — Open in browser / Save as /
    // Add custom / Remove custom. Mirrors the v2_library_* + native
    // dialog flow Tauri uses on artist portraits + album covers.
    window
        .global::<ArtworkActions>()
        .on_open_in_browser(|url| {
            if url.is_empty() {
                return;
            }
            if let Err(e) = open::that(url.as_str()) {
                log::error!("[qbz-slint] artwork open-in-browser failed: {e}");
            }
        });
    {
        let handle = tokio_rt.handle().clone();
        window
            .global::<ArtworkActions>()
            .on_save_as(move |url, default_name| {
                if url.is_empty() {
                    return;
                }
                let url = url.to_string();
                let default = default_name.to_string();
                handle.spawn(async move {
                    let Some(dest) = rfd::AsyncFileDialog::new()
                        .set_file_name(&default)
                        .add_filter("Images", &["jpg", "jpeg", "png"])
                        .save_file()
                        .await
                    else {
                        return;
                    };
                    // Offline: serve the shared disk-cache copy; never attempt
                    // the download.
                    if offline_mode::engine().is_offline() {
                        match artwork::cached_path_for(&url) {
                            Some(path) => {
                                if let Err(e) = tokio::fs::copy(&path, dest.path()).await {
                                    log::error!(
                                        "[qbz-slint] artwork save-as offline copy: {e}"
                                    );
                                }
                            }
                            None => log::warn!(
                                "[qbz-slint] artwork save-as skipped offline: not in disk cache"
                            ),
                        }
                        return;
                    }
                    let bytes = match reqwest::get(&url).await {
                        Ok(resp) => match resp.bytes().await {
                            Ok(b) => b,
                            Err(e) => {
                                log::error!(
                                    "[qbz-slint] artwork save-as fetch body: {e}"
                                );
                                return;
                            }
                        },
                        Err(e) => {
                            log::error!("[qbz-slint] artwork save-as request: {e}");
                            return;
                        }
                    };
                    if let Err(e) = tokio::fs::write(dest.path(), &bytes).await {
                        log::error!("[qbz-slint] artwork save-as write: {e}");
                    }
                });
            });
    }
}
