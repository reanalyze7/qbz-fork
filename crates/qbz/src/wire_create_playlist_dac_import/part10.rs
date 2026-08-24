use crate::*;

pub(crate) fn wire_create_playlist_dac_import_part10(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, _image_cache: &artwork::ImageCache, _settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        // Playlist card actions: play / play-next / queue / share / favorite.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<FavoritesActions>()
            .on_playlist_action(move |id, action| {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                match action.as_str() {
                    "share" => share::copy_to_clipboard(share::qobuz_playlist_url(&id)),
                    "favorite" => {
                        let Ok(pid) = id.parse::<u64>() else {
                            return;
                        };
                        let in_playlists_tab = w
                            .global::<FavoritesState>()
                            .get_active_tab()
                            .to_string()
                            == "playlists";
                        if in_playlists_tab {
                            // Favorites › Playlists: Library sub-tab un-favorites
                            // in place (drop the row); Following sub-tab adds to
                            // the local Library (per user decision).
                            let library = w
                                .global::<FavoritesState>()
                                .get_playlists_sub_tab()
                                .to_string()
                                != "following";
                            let fav = !library;
                            handle.spawn_blocking(move || {
                                crate::library_db::with_db(|db| db.set_playlist_favorite(pid, fav));
                            });
                            if library {
                                favorites::remove_playlist_row(&w, &id);
                            }
                        } else {
                            // Library "All" (mixed feed): authoritative toggle by
                            // the DB state — a foreign card can't know it, and the
                            // owned-but-unhearted case must ADD, not remove.
                            playlist_toggle_favorite_by_id(handle.clone(), weak.clone(), pid, false);
                        }
                    }
                    "follow" => {
                        if let Ok(pid) = id.parse::<u64>() {
                            playlist_set_follow_by_id(
                                runtime.clone(),
                                weak.clone(),
                                handle.clone(),
                                pid,
                                true,
                                false,
                            );
                        }
                    }
                    "unfollow" => {
                        if let Ok(pid) = id.parse::<u64>() {
                            playlist_set_follow_by_id(
                                runtime.clone(),
                                weak.clone(),
                                handle.clone(),
                                pid,
                                false,
                                false,
                            );
                            // In the Favorites › Playlists Following sub-tab,
                            // unfollowing removes the row (mirrors un-favorite).
                            let fs = w.global::<FavoritesState>();
                            if fs.get_active_tab().to_string() == "playlists"
                                && fs.get_playlists_sub_tab().to_string() == "following"
                            {
                                favorites::remove_playlist_row(&w, &id);
                            }
                        }
                    }
                    "copy" => {
                        if let Ok(pid) = id.parse::<u64>() {
                            playlist_copy_by_id(runtime.clone(), weak.clone(), handle.clone(), pid, false);
                        }
                    }
                    act => {
                        // play / play-next / queue: fetch the playlist's tracks,
                        // then play or enqueue.
                        let Ok(pid) = id.parse::<u64>() else {
                            return;
                        };
                        let runtime = runtime.clone();
                        let weak2 = weak.clone();
                        let handle2 = handle.clone();
                        let act = act.to_string();
                        handle.spawn(async move {
                            let tracks = match runtime.core().get_playlist(pid).await {
                                Ok(p) => p.tracks.map(|t| t.items).unwrap_or_default(),
                                Err(e) => {
                                    log::error!("[qbz-slint] playlist {pid} load failed: {e}");
                                    return;
                                }
                            };
                            if tracks.is_empty() {
                                return;
                            }
                            match act.as_str() {
                                "play-next" => {
                                    playback::enqueue_tracks(runtime, handle2, tracks, true)
                                }
                                "queue" => {
                                    playback::enqueue_tracks(runtime, handle2, tracks, false)
                                }
                                _ => {
                                    playback::play_tracks(runtime, weak2, handle2, tracks, 0);
                                }
                            }
                        });
                    }
                }
            });
    }
}
