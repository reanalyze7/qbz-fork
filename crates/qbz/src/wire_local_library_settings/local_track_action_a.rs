use crate::*;

// `on_track_action` arms: play, toggle-select, play-next/queue,
// add-to-playlist. Called unconditionally alongside
// `local_track_action_b` from the single `on_track_action` registration
// (part8.rs) — safe since each action matches at most one of the two, the
// other falls through its own `_ => {}`.
pub(crate) fn local_track_action_a(
    id: &str,
    action: &str,
    runtime: &Arc<AppRuntime<SlintAdapter>>,
    weak: &slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
) {
    let runtime = runtime.clone();
    let weak = weak.clone();
    let handle = handle.clone();
    let id = id.to_string();
    match action {
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
                        let play_next = action == "play-next";
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
        _ => {}
    }
}
