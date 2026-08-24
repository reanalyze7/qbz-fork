use crate::*;

pub(crate) fn wire_local_library_settings_part4(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<LocalAlbumActions>().on_add_to_playlist(move || {
            if let Some(w) = weak.upgrade() {
                let tracks = local_library::current_album_version_tracks(&w);
                let refs: Vec<String> = tracks.iter().map(local_picker_ref).collect();
                if !refs.is_empty() {
                    playlist_picker::open_multi(&w, &refs, true);
                    let runtime = runtime.clone();
                    let weak2 = weak.clone();
                    handle.spawn(async move {
                        let pls = playlist_picker::load(&runtime).await;
                        let _ = weak2.upgrade_in_event_loop(move |w| {
                            playlist_picker::apply(&w, pls);
                        });
                    });
                }
            }
        });
    }
    {
        // Per-row context-menu actions on the local album detail (play-next /
        // queue / add-to-playlist / add-to-mixtape / favorite) — resolved
        // against the open version's track cache; "play" stays on
        // LocalAlbumActions.play-track.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalAlbumActions>()
            .on_track_menu_action(move |id, action| {
                let Some(w) = weak.upgrade() else { return };
                let tracks = local_library::current_album_version_tracks(&w);
                let Some(row) = tracks.iter().find(|t| t.id.to_string() == id.as_str())
                else {
                    return;
                };
                match action.as_str() {
                    "play-next" | "queue" => {
                        playback::enqueue_local_tracks(
                            runtime.clone(),
                            handle.clone(),
                            vec![row.clone()],
                            action.as_str() == "play-next",
                        );
                    }
                    "add-to-playlist" => {
                        playlist_picker::open_multi(&w, &[local_picker_ref(row)], true);
                        let runtime = runtime.clone();
                        let weak2 = weak.clone();
                        handle.spawn(async move {
                            let pls = playlist_picker::load(&runtime).await;
                            let _ = weak2.upgrade_in_event_loop(move |w| {
                                playlist_picker::apply(&w, pls);
                            });
                        });
                    }
                    "add-to-mixtape" => {
                        // Single-row Add to Mixtape/Collection on the local
                        // album detail (spec §3.1) — the row is already
                        // resolved from the open version's track cache.
                        let items =
                            myqbz_add::track_items_from_local(std::slice::from_ref(row));
                        open_add_to_mixtape(weak.clone(), handle.clone(), items);
                    }
                    "favorite" => {
                        // qobuz_download rows only (the menu gates the entry);
                        // toggle by the REAL Qobuz id, never the local row id
                        // (spec §3.2 — Tauri's latent bug, not ported).
                        match row.qobuz_track_id {
                            Some(qid) => toggle_track_favorite(
                                runtime.clone(),
                                weak.clone(),
                                handle.clone(),
                                qid.to_string(),
                            ),
                            None => log::debug!(
                                "[qbz-slint] favorite: album row {id} has no qobuz_track_id"
                            ),
                        }
                    }
                    "go-to-album" | "go-to-artist" => {
                        // Owner improvement over Tauri — source-routed in
                        // local_row_goto. On this surface "Go to album"
                        // reopens the open album for local rows (Qobuz
                        // album-view parity, where the entry also exists);
                        // qobuz_download rows reach their REAL Qobuz pages.
                        local_row_goto(
                            runtime.clone(),
                            weak.clone(),
                            &handle,
                            row.clone(),
                            action.as_str() == "go-to-artist",
                        );
                    }
                    _ => {
                        log::debug!(
                            "[qbz-slint] unhandled local album track action: {id} {action}"
                        );
                    }
                }
            });
    }
}
