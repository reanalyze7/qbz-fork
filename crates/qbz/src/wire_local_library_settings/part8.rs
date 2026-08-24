use crate::*;

// SPLIT-EXCEPTION (crates/qbz/src/main.rs refactor): this fn wraps ONE
// original fn main() statement (a single Slint callback registration or
// startup step) too internally sequential/closure-heavy to decompose
// further without a compiler in the loop (no `cargo check` is permitted
// for this refactor). Left whole, over the 130-line rule, as the
// documented rare exception it allows for.
pub(crate) fn wire_local_library_settings_part8(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalLibraryActions>()
            .on_track_action(move |id, action| {
                match action.as_str() {
                    "play" => {
                        if let Ok(row_id) = id.parse::<i64>() {
                            // Queue the already-loaded rows (instant — no DB
                            // re-query / cover-fill that delayed the queue) so
                            // playback continues down the list from the click.
                            let tracks = local_library::tracks_current_snapshot();
                            if !tracks.is_empty() {
                                let start = tracks
                                    .iter()
                                    .position(|t| t.id == row_id)
                                    .unwrap_or(0);
                                playback::play_local_tracks(
                                    runtime.clone(),
                                    weak.clone(),
                                    handle.clone(),
                                    tracks,
                                    start,
                                    false,
                                );
                            }
                        }
                    }
                    "toggle-select" => {
                        if let Some(w) = weak.upgrade() {
                            local_library::toggle_track_select(&w, id.as_str());
                        }
                    }
                    "play-next" | "queue" => {
                        // Resolve the row from the loaded cache (no DB) and
                        // enqueue; folder-detail rows aren't in the Tracks
                        // cache, so fall back to a DB resolve off-thread.
                        let play_next = action.as_str() == "play-next";
                        if let Some(row) = local_library::local_track_by_id(id.as_str()) {
                            playback::enqueue_local_tracks(
                                runtime.clone(),
                                handle.clone(),
                                vec![row],
                                play_next,
                            );
                        } else if let Ok(rid) = id.parse::<i64>() {
                            let runtime = runtime.clone();
                            let handle2 = handle.clone();
                            handle.spawn(async move {
                                let row = tokio::task::spawn_blocking(move || {
                                    crate::library_db::with_db(|db| db.get_track(rid))
                                        .flatten()
                                })
                                .await
                                .ok()
                                .flatten();
                                if let Some(row) = row {
                                    playback::enqueue_local_tracks(
                                        runtime,
                                        handle2,
                                        vec![row],
                                        play_next,
                                    );
                                }
                            });
                        }
                    }
                    "add-to-playlist" => {
                        // Per-row picker (Tracks tab + folder-detail rows).
                        // Row ids are resolved source-aware at insert, so a folder row
                        // missing from the Tracks cache still works.
                        let Some(w) = weak.upgrade() else { return };
                        let track_ref = match local_library::local_track_by_id(id.as_str()) {
                            Some(row) => local_picker_ref(&row),
                            None => id.to_string(),
                        };
                        playlist_picker::open_multi(&w, &[track_ref], true);
                        let runtime = runtime.clone();
                        let weak2 = weak.clone();
                        handle.spawn(async move {
                            let playlists = playlist_picker::load(&runtime).await;
                            let _ = weak2.upgrade_in_event_loop(move |w| {
                                playlist_picker::apply(&w, playlists);
                            });
                        });
                    }
                    "add-to-mixtape" => {
                        // Single-row Add to Mixtape/Collection (Tracks tab +
                        // folder-detail rows; spec §3.1). Same resolution as
                        // play-next: loaded cache first, DB fallback
                        // off-thread for folder rows.
                        if let Some(row) = local_library::local_track_by_id(id.as_str()) {
                            let items = myqbz_add::track_items_from_local(&[row]);
                            open_add_to_mixtape(weak.clone(), handle.clone(), items);
                        } else if let Ok(rid) = id.parse::<i64>() {
                            let weak2 = weak.clone();
                            let handle2 = handle.clone();
                            handle.spawn(async move {
                                let row = tokio::task::spawn_blocking(move || {
                                    crate::library_db::with_db(|db| db.get_track(rid))
                                        .flatten()
                                })
                                .await
                                .ok()
                                .flatten();
                                if let Some(row) = row {
                                    let items = myqbz_add::track_items_from_local(&[row]);
                                    open_add_to_mixtape(weak2, handle2, items);
                                }
                            });
                        }
                    }
                    "favorite" => {
                        // Library-surface favorite: the menu only shows the
                        // entry on qobuz_download rows (TrackRow gates on
                        // source == "qobuz"), and the toggle uses the row's
                        // REAL qobuz_track_id — never the local row id, which
                        // is what Tauri sends (spec §3.2 latent bug; we port
                        // the intent, not the bug).
                        if let Some(row) = local_library::local_track_by_id(id.as_str()) {
                            match row.qobuz_track_id {
                                Some(qid) => toggle_track_favorite(
                                    runtime.clone(),
                                    weak.clone(),
                                    handle.clone(),
                                    qid.to_string(),
                                ),
                                None => log::debug!(
                                    "[qbz-slint] favorite: local row {id} has no qobuz_track_id"
                                ),
                            }
                        } else if let Ok(rid) = id.parse::<i64>() {
                            // Folder rows aren't in the Tracks cache: resolve
                            // off-thread, then hop back to the UI thread (the
                            // toggle reads/writes UI models).
                            let runtime = runtime.clone();
                            let weak2 = weak.clone();
                            let handle2 = handle.clone();
                            handle.spawn(async move {
                                let row = tokio::task::spawn_blocking(move || {
                                    crate::library_db::with_db(|db| db.get_track(rid))
                                        .flatten()
                                })
                                .await
                                .ok()
                                .flatten();
                                let Some(qid) = row.and_then(|r| r.qobuz_track_id) else {
                                    log::debug!(
                                        "[qbz-slint] favorite: row {rid} has no qobuz_track_id"
                                    );
                                    return;
                                };
                                let weak3 = weak2.clone();
                                let _ = weak2.upgrade_in_event_loop(move |_w| {
                                    toggle_track_favorite(
                                        runtime,
                                        weak3,
                                        handle2,
                                        qid.to_string(),
                                    );
                                });
                            });
                        }
                    }
                    "go-to-album" | "go-to-artist" => {
                        // Owner improvement over Tauri (which omits both on
                        // local rows): resolve the row (Tracks cache first,
                        // DB fallback for folder-detail rows — same seam as
                        // favorite) and source-route in local_row_goto
                        // (local -> local album view / LocalLibrary
                        // artist by name; qobuz_download -> the REAL Qobuz
                        // pages via its qobuz_track_id).
                        let to_artist = action.as_str() == "go-to-artist";
                        if let Some(row) = local_library::local_track_by_id(id.as_str()) {
                            local_row_goto(runtime.clone(), weak.clone(), &handle, row, to_artist);
                        } else if let Ok(rid) = id.parse::<i64>() {
                            let runtime = runtime.clone();
                            let weak2 = weak.clone();
                            let handle2 = handle.clone();
                            handle.spawn(async move {
                                let row = tokio::task::spawn_blocking(move || {
                                    crate::library_db::with_db(|db| db.get_track(rid))
                                        .flatten()
                                })
                                .await
                                .ok()
                                .flatten();
                                match row {
                                    Some(row) => local_row_goto(
                                        runtime, weak2, &handle2, row, to_artist,
                                    ),
                                    None => log::debug!(
                                        "[qbz-slint] go-to: local row {rid} not found"
                                    ),
                                }
                            });
                        }
                    }
                    _ => {
                        log::debug!("[qbz-slint] unhandled local track action: {id} {action}");
                    }
                }
            });
    }
}
