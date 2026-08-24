use crate::*;

pub(crate) fn wire_queue_and_cards_part5(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {

    // Dedicated discography page — infinite load-more.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<ArtistReleasesActions>()
            .on_load_more(move || {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                let st = w.global::<ArtistReleasesState>();
                if st.get_load_more_loading() || !st.get_has_more() {
                    return;
                }
                let artist_id = st.get_id().to_string();
                let release_type = st.get_release_type().to_string();
                if artist_id.is_empty() {
                    return;
                }
                let offset = artist_releases::loaded_count(&w);
                st.set_load_more_loading(true);
                let runtime = runtime.clone();
                let weak2 = weak.clone();
                let image_cache = image_cache.clone();
                handle.spawn(async move {
                    match artist::load_release_page(&runtime, &artist_id, &release_type, offset)
                        .await
                    {
                        Ok((cards, has_more)) => {
                            let image_cache = image_cache.clone();
                            let _ = weak2.upgrade_in_event_loop(move |w| {
                                let jobs = artist_releases::apply_page(&w, cards, has_more, false);
                                artwork::spawn_loads(jobs, w.as_weak(), image_cache);
                            });
                        }
                        Err(e) => {
                            log::warn!("[qbz-slint] artist releases load-more failed: {e}");
                            let _ = weak2.upgrade_in_event_loop(|w| {
                                w.global::<ArtistReleasesState>().set_load_more_loading(false);
                            });
                        }
                    }
                });
            });
    }

    // Dedicated discography page — sort change (persisted, shared with index).
    {
        let weak = window.as_weak();
        window
            .global::<ArtistReleasesActions>()
            .on_set_sort(move |sort| {
                if let Some(w) = weak.upgrade() {
                    artist_releases::resort(&w, sort.as_str());
                }
            });
    }

    // Dedicated discography page — retry after a failed load.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<ArtistReleasesActions>()
            .on_retry(move || {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                let st = w.global::<ArtistReleasesState>();
                let artist_id = st.get_id().to_string();
                let name = st.get_name().to_string();
                let release_type = st.get_release_type().to_string();
                if artist_id.is_empty() {
                    return;
                }
                navigate_artist_releases(
                    runtime.clone(),
                    weak.clone(),
                    &handle,
                    image_cache.clone(),
                    artist_id,
                    name,
                    release_type,
                );
            });
    }

    // Artist Popular Tracks multi-select — the section toggle.
    {
        let weak = window.as_weak();
        window
            .global::<ArtistActions>()
            .on_toggle_top_tracks_select(move || {
                if let Some(w) = weak.upgrade() {
                    let on = w.global::<ArtistState>().get_top_tracks_multi_select();
                    artist::set_multi_select(&w, !on);
                }
            });
    }
}
