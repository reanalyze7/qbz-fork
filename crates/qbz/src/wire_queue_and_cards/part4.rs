use crate::*;

pub(crate) fn wire_queue_and_cards_part4(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, _settings_ctx: &Arc<settings::SettingsCtx>) {

    // Artist per-section sort (persisted by release_type).
    {
        let weak = window.as_weak();
        window
            .global::<ArtistActions>()
            .on_set_section_sort(move |rt, sort| {
                if let Some(w) = weak.upgrade() {
                    artist::resort_section(&w, rt.as_str(), sort.as_str());
                }
            });
    }

    // Artist per-section load-more (capped to 4 pages; reuses get_releases_grid).
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<ArtistActions>()
            .on_load_more_section(move |rt| {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                let release_type = rt.to_string();
                if !artist::section_can_load_more(&release_type) {
                    return;
                }
                let artist_id = w.global::<ArtistState>().get_id().to_string();
                if artist_id.is_empty() {
                    return;
                }
                let offset = artist::section_loaded_count(&w, &release_type) as u32;
                let runtime = runtime.clone();
                let weak2 = weak.clone();
                let image_cache = image_cache.clone();
                handle.spawn(async move {
                    match artist::load_release_page(&runtime, &artist_id, &release_type, offset)
                        .await
                    {
                        Ok((cards, has_more)) => {
                            let image_cache = image_cache.clone();
                            let rt2 = release_type.clone();
                            let _ = weak2.upgrade_in_event_loop(move |w| {
                                let jobs =
                                    artist::append_release_page(&w, &rt2, cards, has_more);
                                artwork::spawn_loads(jobs, w.as_weak(), image_cache);
                            });
                        }
                        Err(e) => {
                            log::warn!("[qbz-slint] load-more {release_type} failed: {e}")
                        }
                    }
                });
            });
    }

    // Artist "See discography" — open the dedicated releases page pre-filtered.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<ArtistActions>()
            .on_open_releases(move |rt| {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                let artist_id = w.global::<ArtistState>().get_id().to_string();
                let artist_name = w.global::<ArtistState>().get_name().to_string();
                if artist_id.is_empty() {
                    return;
                }
                let release_type = rt.to_string();
                nav::record(nav::NavEntry::ArtistReleases {
                    id: artist_id.clone(),
                    name: artist_name.clone(),
                    release_type: release_type.clone(),
                });
                navigate_artist_releases(
                    runtime.clone(),
                    weak.clone(),
                    &handle,
                    image_cache.clone(),
                    artist_id,
                    artist_name,
                    release_type,
                );
                update_nav_flags(&w);
            });
    }
}
