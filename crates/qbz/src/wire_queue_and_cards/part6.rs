use crate::*;

pub(crate) fn wire_queue_and_cards_part6(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {

    // Artist Popular Tracks bulk bar — actions over the selected rows.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<ArtistActions>()
            .on_top_tracks_bulk_action(move |action| {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                let artist_id = w.global::<ArtistState>().get_id().to_string();
                match action.as_str() {
                    "select-all" => artist::select_all(&w),
                    "clear" => artist::clear_selection(&w),
                    "play-next" => playback::enqueue_artist_top_selected(
                        runtime.clone(),
                        weak.clone(),
                        handle.clone(),
                        artist_id,
                        artist::selected_ids(&w),
                        true,
                    ),
                    "queue" => playback::enqueue_artist_top_selected(
                        runtime.clone(),
                        weak.clone(),
                        handle.clone(),
                        artist_id,
                        artist::selected_ids(&w),
                        false,
                    ),
                    "add-to-playlist" => {
                        let ids = artist::selected_ids(&w);
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
                    "add-to-favorites" => {
                        let ids = artist::selected_ids(&w);
                        if ids.is_empty() {
                            return;
                        }
                        let runtime = runtime.clone();
                        let weak = weak.clone();
                        handle.spawn(async move {
                            for id in &ids {
                                match runtime.core().add_favorite("track", id).await {
                                    Ok(()) => {
                                        if let Ok(tid) = id.parse::<u64>() {
                                            crate::fav_cache::set(tid, true);
                                        }
                                    }
                                    Err(e) => log::error!(
                                        "[qbz-slint] bulk favorite track {id} failed: {e}"
                                    ),
                                }
                            }
                            let _ = weak.upgrade_in_event_loop(|w| {
                                artist::clear_selection(&w);
                                crate::toast::success(&w, "Added to favorites");
                            });
                        });
                    }
                    "add-to-mixtape" => {
                        let items = mixtape_items_from_artist_selection(&w);
                        if !items.is_empty() {
                            open_add_to_mixtape(weak.clone(), handle.clone(), items);
                            artist::clear_selection(&w);
                        }
                    }
                    _ => {}
                }
            });
    }
}
