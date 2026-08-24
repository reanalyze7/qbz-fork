use crate::*;

pub(crate) fn wire_create_playlist_dac_import_part9(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, _settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        // Labels "random" — open a random visible label's landing.
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_labels_random(move || {
                if let Some(w) = weak.upgrade() {
                    if let Some((id, name)) = favorites::random_visible_label(&w) {
                        w.global::<FavoritesActions>()
                            .invoke_open_label(id.into(), name.into());
                    }
                }
            });
    }
    {
        // Group the favorite artists (off / A-Z) — persisted.
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_artists_set_group(move |g| {
                if let Some(w) = weak.upgrade() {
                    w.global::<FavoritesState>()
                        .set_artists_group_enabled(g == "alpha");
                    favorites::derive_artists(&w);
                    favorites_prefs::save(&w);
                }
            });
    }
    {
        // Artists grid <-> sidepanel view toggle (persisted). Switching back to
        // grid clears the sidepanel selection (matches Tauri).
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_artists_set_view(move |v| {
                if let Some(w) = weak.upgrade() {
                    let st = w.global::<FavoritesState>();
                    st.set_artists_view_mode(v.clone());
                    if v == "grid" {
                        st.set_selected_artist_id("".into());
                    }
                    // Rebuild grouped/alpha for the new mode (the sidepanel
                    // left list is always grouped).
                    favorites::derive_artists(&w);
                    favorites_prefs::save(&w);
                }
            });
    }
    {
        // Sidepanel: load + show the selected artist's albums, reusing the
        // standalone artist page's /artist/page release_type classifier.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<FavoritesActions>()
            .on_select_artist(move |id, name| {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                let st = w.global::<FavoritesState>();
                st.set_selected_artist_id(id.clone());
                st.set_selected_artist_name(name);
                st.set_selected_albums_loading(true);
                st.set_selected_albums_error("".into());
                let runtime = runtime.clone();
                let weak2 = weak.clone();
                let image_cache = image_cache.clone();
                let id_s = id.to_string();
                handle.spawn(async move {
                    match artist::load_artist(&runtime, &id_s).await {
                        Ok(data) => {
                            let sections = data.release_sections;
                            let jobs = favorites::selected_artist_artwork_jobs(&sections);
                            let _ = weak2.upgrade_in_event_loop(move |w| {
                                favorites::apply_selected_artist(&w, sections);
                            });
                            artwork::spawn_loads(jobs, weak2.clone(), image_cache.clone());
                        }
                        Err(e) => {
                            log::error!("[qbz-slint] sidepanel artist {id_s} load failed: {e}");
                            let _ = weak2.upgrade_in_event_loop(move |w| {
                                let st = w.global::<FavoritesState>();
                                st.set_selected_albums_loading(false);
                                st.set_selected_albums_error(e.into());
                            });
                        }
                    }
                });
            });
    }
}
