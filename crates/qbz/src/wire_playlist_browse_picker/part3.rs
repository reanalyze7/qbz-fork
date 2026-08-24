use crate::*;

// SPLIT-EXCEPTION (crates/qbz/src/main.rs refactor): this fn wraps ONE
// original fn main() statement (a single Slint callback registration or
// startup step) too internally sequential/closure-heavy to decompose
// further without a compiler in the loop (no `cargo check` is permitted
// for this refactor). Left whole, over the 130-line rule, as the
// documented rare exception it allows for.
pub(crate) fn wire_playlist_browse_picker_part3(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {

    // Favorites view actions — tab switch (lazy-load), open album /
    // artist, and per-row track actions routed to the media-action
    // "Add to playlist" picker — pick TOGGLES membership (checkbox
    // semantics, spec PLAYLIST-REDESIGN-SPEC.md §4): not-yet-present adds
    // the pending track(s), already-present removes them. Never closes the
    // picker (only close() does — footer "Done" / backdrop); close
    // dismisses.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<PlaylistPickerActions>()
            .on_pick(move |playlist_id| {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                let picker = w.global::<PlaylistPickerState>();
                let is_local = picker.get_local_mode();
                // Bulk add carries track-ids; single add carries track-id.
                let ids_model = picker.get_track_ids();
                let track_id_single = picker.get_track_id().to_string();
                // Resolve the target name for the success toast.
                let target_name = picker_playlist_name(&w, playlist_id.as_str());

                let already_has = {
                    use slint::Model;
                    let model = picker.get_playlists();
                    (0..model.row_count())
                        .filter_map(|i| model.row_data(i))
                        .find(|item| item.id.as_str() == playlist_id.as_str())
                        .map(|item| item.already_has)
                        .unwrap_or(false)
                };
                if already_has {
                    toggle_off_playlist_pick(
                        &runtime,
                        &weak,
                        &handle,
                        playlist_id.to_string(),
                        target_name,
                        is_local,
                        &ids_model,
                        &track_id_single,
                    );
                    return;
                }

                // --- ADD (unchanged below except the row is no longer
                // closed on pick — see toggle_off_playlist_pick for the
                // remove side) ---
                // LOCAL playlist target (id "local:<uuid>") — writes go to
                // the library.db repo (works offline; D7 routing).
                if local_playlist::is_local_id(playlist_id.as_str()) {
                    let target = playlist_id.to_string();
                    if is_local {
                        // Local-mode refs — LocalLibrary row ids ("<i64>",
                        // source-aware mapping: local path / offline-copy
                        // Qobuz id).
                        let refs: Vec<String> = (0..ids_model.row_count())
                            .filter_map(|i| ids_model.row_data(i))
                            .map(|s| s.to_string())
                            .collect();
                        if refs.is_empty() {
                            return;
                        }
                        let weak = weak.clone();
                        let tname = target_name.clone();
                        let mark_id = target.clone();
                        handle.spawn(async move {
                            let added = tokio::task::spawn_blocking(move || {
                                local_playlist::add_local_refs_blocking(&target, &refs)
                            })
                            .await
                            .unwrap_or(0);
                            // reco: local refs are not Qobuz catalog ids — not
                            // logged (same source gate as local plays).
                            toast_added_tracks(&weak, added, tname);
                            if added > 0 {
                                let _ = weak.upgrade_in_event_loop(move |w| {
                                    playlist_picker::mark_row_already_has(&w, &mark_id, true);
                                });
                            }
                        });
                        return;
                    }
                    let mut ids: Vec<u64> = (0..ids_model.row_count())
                        .filter_map(|i| ids_model.row_data(i))
                        .filter_map(|s| s.parse::<u64>().ok())
                        .collect();
                    if ids.is_empty() {
                        if let Ok(tid) = track_id_single.parse::<u64>() {
                            ids.push(tid);
                        }
                    }
                    if ids.is_empty() {
                        return;
                    }
                    let weak = weak.clone();
                    let tname = target_name.clone();
                    let mark_id = target.clone();
                    handle.spawn(async move {
                        // reco: keep the full Qobuz ids before they move into
                        // the add closure (local-playlist target = no Qobuz pid).
                        let reco_ids = ids.clone();
                        let added = tokio::task::spawn_blocking(move || {
                            local_playlist::add_qobuz_tracks_blocking(&target, &ids)
                        })
                        .await
                        .unwrap_or(0);
                        tokio::task::spawn_blocking(move || {
                            crate::reco::log_playlist_add(None, reco_ids)
                        });
                        toast_added_tracks(&weak, added, tname);
                        if added > 0 {
                            let _ = weak.upgrade_in_event_loop(move |w| {
                                playlist_picker::mark_row_already_has(&w, &mark_id, true);
                            });
                        }
                    });
                    return;
                }

                let Ok(pid) = playlist_id.parse::<u64>() else {
                    return;
                };

                if is_local {
                    // Local-mode refs onto a QOBUZ playlist: row ids attach
                    // via the local sidecar (same table the offline detail
                    // renders).
                    let refs: Vec<String> = (0..ids_model.row_count())
                        .filter_map(|i| ids_model.row_data(i))
                        .map(|s| s.to_string())
                        .collect();
                    if refs.is_empty() {
                        return;
                    }
                    // Seam C: append after the whole merged list AND past
                    // any stored position (the old 0-based `enumerate`
                    // write collided slots -> silent row loss in the
                    // interleave). Base = the Qobuz block size from the
                    // sidebar's session cache; re-adding an existing ref
                    // MOVES it to the append slot (INSERT OR REPLACE, E4).
                    let qobuz_count = sidebar::playlist_track_count(pid).unwrap_or(0);
                    let refs_count = refs.len();
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    let handle2 = handle.clone();
                    let image_cache = image_cache.clone();
                    let tname = target_name.clone();
                    handle.spawn(async move {
                        let _ = tokio::task::spawn_blocking(move || {
                            crate::library_db::with_db(|db| {
                                let mut next =
                                    db.next_playlist_sidecar_position(pid, qobuz_count)?;
                                for r in refs.iter() {
                                    if let Ok(lid) = r.parse::<i64>() {
                                        db.add_local_track_to_playlist(pid, lid, next)?;
                                        next += 1;
                                    }
                                }
                                Ok(())
                            })
                        })
                        .await;
                        // reco: local refs are not Qobuz catalog ids — not
                        // logged (same source gate as local plays).
                        toast_added_tracks(&weak, refs_count, tname);
                        if refs_count > 0 {
                            let _ = weak.clone().upgrade_in_event_loop(move |w| {
                                playlist_picker::mark_row_already_has(&w, &pid.to_string(), true);
                            });
                        }
                        // E12: the open detail re-merges so the rows show
                        // up immediately.
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
                    });
                    return;
                }

                // Qobuz tracks → Qobuz playlist. Run duplicate detection first
                // (Tauri parity: this is the ONLY branch that checks dupes).
                // If any of the ids are already in the target, park the context
                // in DUP_CONFIRM_STASH and open the confirm sub-modal; the user
                // then chooses add-all / add-new-only. With no dupes we add
                // directly and toast.
                let mut ids: Vec<u64> = (0..ids_model.row_count())
                    .filter_map(|i| ids_model.row_data(i))
                    .filter_map(|s| s.parse::<u64>().ok())
                    .collect();
                if ids.is_empty() {
                    if let Ok(tid) = track_id_single.parse::<u64>() {
                        ids.push(tid);
                    }
                }
                if ids.is_empty() {
                    return;
                }
                let runtime = runtime.clone();
                let weak = weak.clone();
                let tname = target_name.clone();
                handle.spawn(async move {
                    match runtime.core().check_playlist_duplicates(pid, &ids).await {
                        Ok(dup) if dup.duplicate_count > 0 => {
                            // Stash the full context; the confirm handlers read
                            // it back. Open the sub-modal with the counts.
                            let total = dup.total_tracks as i32;
                            let dupc = dup.duplicate_count as i32;
                            let dup_ids = dup.duplicate_track_ids.clone();
                            let stash = (pid, ids.clone(), dup_ids, tname.clone());
                            let _ = weak.upgrade_in_event_loop(move |w| {
                                DUP_CONFIRM_STASH.with(|c| *c.borrow_mut() = Some(stash));
                                let st = w.global::<DuplicateConfirmState>();
                                st.set_duplicate_count(dupc);
                                st.set_total_count(total);
                                st.set_busy(false);
                                st.set_playlist_name(tname.into());
                                st.set_open(true);
                            });
                        }
                        Ok(_) => {
                            // No duplicates — add directly + toast.
                            let n = ids.len();
                            if let Err(e) =
                                runtime.core().add_tracks_to_playlist(pid, &ids).await
                            {
                                log::error!("[qbz-slint] add to playlist failed: {e}");
                            } else {
                                // reco: log the full requested Qobuz ids.
                                let reco_ids = ids.clone();
                                tokio::task::spawn_blocking(move || {
                                    crate::reco::log_playlist_add(Some(pid), reco_ids)
                                });
                                toast_added_tracks(&weak, n, tname);
                                let _ = weak.upgrade_in_event_loop(move |w| {
                                    playlist_picker::mark_row_already_has(&w, &pid.to_string(), true);
                                });
                            }
                        }
                        Err(e) => {
                            // Dup check failed (e.g. transient API) — fall back
                            // to a direct add so the action still completes.
                            log::warn!(
                                "[qbz-slint] dup check failed, adding directly: {e}"
                            );
                            let n = ids.len();
                            if let Err(e) =
                                runtime.core().add_tracks_to_playlist(pid, &ids).await
                            {
                                log::error!("[qbz-slint] add to playlist failed: {e}");
                            } else {
                                // reco: log the full requested Qobuz ids.
                                let reco_ids = ids.clone();
                                tokio::task::spawn_blocking(move || {
                                    crate::reco::log_playlist_add(Some(pid), reco_ids)
                                });
                                toast_added_tracks(&weak, n, tname);
                                let _ = weak.upgrade_in_event_loop(move |w| {
                                    playlist_picker::mark_row_already_has(&w, &pid.to_string(), true);
                                });
                            }
                        }
                    }
                });
            });
    }
}
