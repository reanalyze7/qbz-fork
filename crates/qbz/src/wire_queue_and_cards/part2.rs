use crate::*;

pub(crate) fn wire_queue_and_cards_part2(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {

    // Album multi-select bulk bar — actions over the selected catalog rows.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<AlbumActions>()
            .on_bulk_action(move |action| {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                match action.as_str() {
                    "select-all" => album::select_all(&w),
                    "clear" => album::clear_selection(&w),
                    "queue" => {
                        let tracks = album::selected_play_tracks(&w);
                        if !tracks.is_empty() {
                            playback::enqueue_tracks(
                                runtime.clone(),
                                handle.clone(),
                                tracks,
                                false,
                            );
                        }
                    }
                    "play-next" => {
                        let tracks = album::selected_play_tracks(&w);
                        if !tracks.is_empty() {
                            playback::enqueue_tracks(
                                runtime.clone(),
                                handle.clone(),
                                tracks,
                                true,
                            );
                        }
                    }
                    "make-offline" => {
                        let tracks = album::selected_play_tracks(&w);
                        if !tracks.is_empty() {
                            offline_cache::cache_tracks(
                                runtime.clone(),
                                weak.clone(),
                                handle.clone(),
                                tracks,
                            );
                            album::clear_selection(&w);
                        }
                    }
                    "add-to-playlist" => {
                        let ids = album::selected_ids(&w);
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
                        let ids = album::selected_ids(&w);
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
                                album::clear_selection(&w);
                                crate::toast::success(&w, "Added to favorites");
                            });
                        });
                    }
                    _ => {}
                }
            });
    }
}
