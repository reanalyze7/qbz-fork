use crate::*;

pub(crate) fn wire_playlist_crud_sidebar_part2(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window.global::<DragActions>().on_end(move || {
            let Some(w) = weak.upgrade() else { return };
            let ds = w.global::<DragState>();
            let pid = ds.get_over_playlist_id().to_string();
            ds.set_active(false);
            ds.set_over_playlist_id("".into());
            let tracks = drag::dragged();
            drag::clear();
            if tracks.is_empty() {
                return;
            }
            // Drop onto a LOCAL playlist row — write the repo source-aware
            // (D7 routing): local file rows store local_path,
            // Qobuz/offline-cached rows qobuz_track_id.
            if local_playlist::is_local_id(&pid) {
                handle.spawn(async move {
                    let n = tokio::task::spawn_blocking(move || {
                        local_playlist::add_drag_tracks_blocking(&pid, &tracks)
                    })
                    .await
                    .unwrap_or(0);
                    log::info!("[qbz-slint] dropped {n} track(s) onto local playlist");
                });
                return;
            }
            if let Ok(pid) = pid.parse::<u64>() {
                // Qobuz playlist target: catalog ids become real membership;
                // local rows attach via the mixed-playlist sidecar (the
                // same table the picker's local mode writes).
                let mut qobuz: Vec<u64> = Vec::new();
                let mut local_rows: Vec<i64> = Vec::new();
                for item in tracks {
                    match item {
                        drag::DragTrack::Qobuz(id) => qobuz.push(id),
                        drag::DragTrack::LocalRow(id) => local_rows.push(id),
                    }
                }
                let runtime = runtime.clone();
                let weak = weak.clone();
                let handle2 = handle.clone();
                let image_cache = image_cache.clone();
                handle.spawn(async move {
                    let mut added = 0usize;
                    if !qobuz.is_empty() {
                        match runtime.core().add_tracks_to_playlist(pid, &qobuz).await {
                            Ok(()) => added += qobuz.len(),
                            Err(e) => {
                                log::error!("[qbz-slint] drop add to playlist failed: {e}")
                            }
                        }
                    }
                    let sidecar_added = !local_rows.is_empty();
                    if sidecar_added {
                        // Seam C: append after the merged list / past any
                        // stored position — never 0-based. The Qobuz block
                        // size includes the rows the SAME drop just added
                        // (the sidebar cache hasn't seen them yet).
                        let qobuz_count = sidebar::playlist_track_count(pid)
                            .unwrap_or(0)
                            + qobuz.len() as u32;
                        let n = tokio::task::spawn_blocking(move || {
                            crate::library_db::with_db(|db| {
                                let mut next =
                                    db.next_playlist_sidecar_position(pid, qobuz_count)?;
                                for rid in local_rows.iter() {
                                    db.add_local_track_to_playlist(pid, *rid, next)?;
                                    next += 1;
                                }
                                Ok(local_rows.len())
                            })
                            .unwrap_or(0)
                        })
                        .await
                        .unwrap_or(0);
                        added += n;
                    }
                    if added > 0 {
                        log::info!(
                            "[qbz-slint] dropped {added} track(s) onto playlist {pid}"
                        );
                    }
                    if sidecar_added {
                        // E12: re-merge the open detail after a sidecar
                        // write to the same playlist.
                        let _ = weak.clone().upgrade_in_event_loop(move |w| {
                            if w.global::<NavState>().get_view() == ContentView::Playlist
                                && w.global::<PlaylistState>().get_id().to_string()
                                    == pid.to_string()
                            {
                                navigate_playlist(
                                    runtime,
                                    weak,
                                    &handle2,
                                    image_cache,
                                    pid.to_string(),
                                );
                            }
                        });
                    }
                });
            }
        });
    }
}
