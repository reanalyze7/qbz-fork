use crate::*;

pub(crate) fn wire_queue_and_cards_part7(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {

    // Artist Popular Tracks section "more" menu — all-tracks actions.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<ArtistActions>()
            .on_top_tracks_menu_action(move |action| {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                let artist_id = w.global::<ArtistState>().get_id().to_string();
                if artist_id.is_empty() {
                    return;
                }
                match action.as_str() {
                    "next-all" => playback::enqueue_artist_top_selected(
                        runtime.clone(),
                        weak.clone(),
                        handle.clone(),
                        artist_id,
                        artist::all_top_track_ids(&w),
                        true,
                    ),
                    "queue-all" => playback::enqueue_artist_top_selected(
                        runtime.clone(),
                        weak.clone(),
                        handle.clone(),
                        artist_id,
                        artist::all_top_track_ids(&w),
                        false,
                    ),
                    "shuffle-all" => playback::play_artist_top_shuffled(
                        runtime.clone(),
                        weak.clone(),
                        handle.clone(),
                        artist_id,
                    ),
                    "playlist-all" => {
                        let ids = artist::all_top_track_ids(&w);
                        if !ids.is_empty() {
                            playlist_picker::open_multi(&w, &ids, false);
                            let runtime = runtime.clone();
                            let weak = weak.clone();
                            handle.spawn(async move {
                                let playlists = playlist_picker::load(&runtime).await;
                                let _ = weak.upgrade_in_event_loop(move |w| {
                                    playlist_picker::apply(&w, playlists);
                                });
                            });
                        }
                    }
                    _ => {}
                }
            });
    }

    // Artist network sidebar — no persistence. Default open, user can
    // close per-session, and reset_network_sidebar re-applies the open
    // state on every artist navigation (open unless the content area is
    // space-constrained — see reset_network_sidebar). The toggle
    // callback stays a no-op on the Rust side — Slint already flips
    // NetworkSidebarState.open directly in the click handler.
    window
        .global::<NetworkSidebarActions>()
        .on_toggle(|| {});

    // Network sidebar — typed click callbacks. Each delivers the
    // minimum payload the future target views (ArtistsByLocation,
    // LabelReleases, MusicianPage) will need. Logged-only until those
    // views land in Slint.
    // Location click — open ArtistsByLocationView using the cached
    // location params from the Origin metadata (area, genres, tags).
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<NetworkSidebarActions>()
            .on_location_clicked(move |mbid| {
                let Some(params) = artist::location_params() else {
                    log::warn!(
                        "[qbz-slint] location clicked but no cached params (mbid={mbid})"
                    );
                    return;
                };
                nav::record(nav::NavEntry::Location {
                    mbid: params.mbid.clone(),
                    area_id: params.area_id.clone(),
                    area_name: params.area_name.clone(),
                    country: params.country.clone(),
                    genres: params.genres.clone(),
                    tags: params.tags.clone(),
                });
                navigate_location(
                    runtime.clone(),
                    weak.clone(),
                    &handle,
                    image_cache.clone(),
                    params,
                );
                if let Some(w) = weak.upgrade() {
                    update_nav_flags(&w);
                }
            });
    }
}
