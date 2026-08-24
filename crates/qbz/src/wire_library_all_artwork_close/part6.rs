use crate::*;

pub(crate) fn wire_library_all_artwork_close_part6(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        let handle = tokio_rt.handle().clone();
        let weak = window.as_weak();
        let image_cache = image_cache.clone();
        window
            .global::<ArtworkActions>()
            .on_add_custom(move |kind, key| {
                let kind = kind.to_string();
                let key = key.to_string();
                let weak = weak.clone();
                let image_cache = image_cache.clone();
                handle.spawn(async move {
                    let Some(file) = rfd::AsyncFileDialog::new()
                        .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
                        .pick_file()
                        .await
                    else {
                        return;
                    };
                    let path = file.path().to_string_lossy().into_owned();
                    match kind.as_str() {
                        "artist" => {
                            custom_artwork::set_artist_image(&key, &path);
                            // Decode + apply immediately so the new image shows
                            // without a reload — critical for artists with no
                            // Qobuz portrait (e.g. Vicky Psarakis), where there
                            // is no network artwork to fall back on.
                            let decoded = artwork::fetch_and_decode_ref(
                                &qbz_models::ArtworkRef::LocalFile(path.clone()),
                                &image_cache,
                                440,
                            )
                            .await;
                            let _ = weak.upgrade_in_event_loop(move |w| {
                                w.global::<ArtistState>().set_has_custom_image(true);
                                if let Some((pixels, iw, ih)) = decoded {
                                    artist::apply_artwork(&w, &pixels, iw, ih);
                                }
                            });
                        }
                        "album" => {
                            custom_artwork::set_album_cover(&key, &path);
                            let decoded = artwork::fetch_and_decode_ref(
                                &qbz_models::ArtworkRef::LocalFile(path.clone()),
                                &image_cache,
                                448,
                            )
                            .await;
                            let _ = weak.upgrade_in_event_loop(move |w| {
                                w.global::<AlbumState>().set_has_custom_cover(true);
                                if let Some((pixels, iw, ih)) = decoded {
                                    album::apply_artwork(&w, &pixels, iw, ih);
                                }
                            });
                        }
                        _ => log::warn!(
                            "[qbz-slint] artwork add-custom: unknown kind {kind}"
                        ),
                    }
                });
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<ArtworkActions>()
            .on_remove_custom(move |kind, key| {
                match kind.as_str() {
                    "artist" => {
                        custom_artwork::remove_artist_image(key.as_str());
                        if let Some(w) = weak.upgrade() {
                            w.global::<ArtistState>().set_has_custom_image(false);
                        }
                    }
                    "album" => {
                        custom_artwork::remove_album_cover(key.as_str());
                        if let Some(w) = weak.upgrade() {
                            w.global::<AlbumState>().set_has_custom_cover(false);
                        }
                    }
                    _ => log::warn!(
                        "[qbz-slint] artwork remove-custom: unknown kind {kind}"
                    ),
                }
            });
    }

    window.on_close_app({
        let weak = window.as_weak();
        move || {
            // Custom titlebar close button. Hide to tray when close-to-tray is
            // enabled and the tray is live; otherwise quit.
            if tray_settings::get().close_to_tray && tray::handle().is_some() {
                log::info!("[qbz-slint] close-to-tray (titlebar): hiding to tray");
                // Flush the session even when only hiding — the process may be
                // killed from the tray / shell without a real quit afterwards.
                session_persist::save_on_exit();
                tray::hide_window(&weak);
            } else {
                log::info!("[qbz-slint] closing");
                // Flush the final session snapshot before quitting.
                session_persist::save_on_exit();
                let _ = slint::quit_event_loop();
            }
        }
    });
}
