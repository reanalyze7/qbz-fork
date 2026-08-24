use crate::*;

// SPLIT-EXCEPTION (crates/qbz/src/main.rs refactor): this fn wraps ONE
// original fn main() statement (a single Slint callback registration or
// startup step) too internally sequential/closure-heavy to decompose
// further without a compiler in the loop (no `cargo check` is permitted
// for this refactor). Left whole, over the 130-line rule, as the
// documented rare exception it allows for.
pub(crate) fn wire_playlist_browse_picker_part5(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {

    // Inline "Create new playlist" → create-and-add (PlaylistCreateRow).
    // Creates a playlist (Qobuz online / local offline per D8) and adds the
    // carried tracks to it, collapses the create row, and reloads the
    // picker list so the new playlist shows up checked — the picker itself
    // STAYS OPEN (spec §2/§4: only "Done" / backdrop close it). Discriminates
    // the carried ids exactly like the pick handler (local-mode refs vs
    // Qobuz u64 ids).
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<PlaylistPickerActions>()
            .on_create_and_add(move || {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                use slint::Model;
                let picker = w.global::<PlaylistPickerState>();
                let name = picker.get_create_name().to_string();
                if name.trim().is_empty() || picker.get_creating() {
                    return;
                }
                let is_local = picker.get_local_mode();
                let ids_model = picker.get_track_ids();
                let track_id_single = picker.get_track_id().to_string();
                // Local-mode refs (LocalLibrary row ids) for the
                // local-playlist add; Qobuz u64 ids for the online path.
                let refs: Vec<String> = (0..ids_model.row_count())
                    .filter_map(|i| ids_model.row_data(i))
                    .map(|s| s.to_string())
                    .collect();
                let mut qobuz_ids: Vec<u64> = (0..ids_model.row_count())
                    .filter_map(|i| ids_model.row_data(i))
                    .filter_map(|s| s.parse::<u64>().ok())
                    .collect();
                if qobuz_ids.is_empty() {
                    if let Ok(tid) = track_id_single.parse::<u64>() {
                        qobuz_ids.push(tid);
                    }
                }
                picker.set_creating(true);

                let offline_now = offline_mode::engine().is_offline();
                let nm = name.trim().to_string();
                let runtime = runtime.clone();
                let weak = weak.clone();
                let handle2 = handle.clone();

                if offline_now {
                    // D8: offline ⇒ LOCAL playlist (library.db), never the
                    // retired pending-playlist engine. Mirrors the create
                    // modal's offline branch.
                    let local_refs = refs.clone();
                    let local_qobuz = qobuz_ids.clone();
                    // reco: the full Qobuz ids (empty when adding local refs).
                    let reco_qobuz: Vec<u64> = if is_local { Vec::new() } else { qobuz_ids.clone() };
                    handle.spawn(async move {
                        let created = tokio::task::spawn_blocking({
                            let nm = nm.clone();
                            move || local_playlist::create_blocking(&nm, None, true)
                        })
                        .await
                        .ok()
                        .flatten();
                        let mut added = 0usize;
                        if let Some(ref new_id) = created {
                            let new_id = new_id.clone();
                            added = tokio::task::spawn_blocking(move || {
                                if is_local {
                                    local_playlist::add_local_refs_blocking(
                                        &new_id,
                                        &local_refs,
                                    )
                                } else {
                                    local_playlist::add_qobuz_tracks_blocking(
                                        &new_id,
                                        &local_qobuz,
                                    )
                                }
                            })
                            .await
                            .unwrap_or(0);
                        }
                        // reco: log the new playlist's Qobuz tracks (new local
                        // playlist = no Qobuz pid; empty when local refs).
                        if created.is_some() {
                            let reco_ids = reco_qobuz;
                            tokio::task::spawn_blocking(move || {
                                crate::reco::log_playlist_add(None, reco_ids)
                            });
                        }
                        let r2 = runtime.clone();
                        let h2 = handle2.clone();
                        let weak2 = weak.clone();
                        let nm2 = nm.clone();
                        let _ = weak.upgrade_in_event_loop(move |w| {
                            let st = w.global::<PlaylistPickerState>();
                            st.set_creating(false);
                            st.set_creating_open(false);
                            st.set_create_name("".into());
                            // Stays open (spec §2: only the footer "Done" /
                            // backdrop close it) — reload so the new playlist
                            // appears, checked if tracks were carried into it.
                            match created {
                                Some(_) => {
                                    toast_added_tracks(&weak2, added, nm2);
                                    load_sidebar_playlists(r2.clone(), weak2.clone(), &h2);
                                    h2.spawn(async move {
                                        let playlists = playlist_picker::load(&r2).await;
                                        let _ = weak2.upgrade_in_event_loop(move |w| {
                                            playlist_picker::apply(&w, playlists)
                                        });
                                    });
                                }
                                None => {
                                    log::error!(
                                        "[qbz-slint] create-and-add (local) failed"
                                    );
                                }
                            }
                        });
                    });
                    return;
                }

                // Online ⇒ Qobuz playlist, then add the carried tracks.
                handle.spawn(async move {
                    match runtime.core().create_playlist(&nm, None, false).await {
                        Ok(playlist) => {
                            let pid = playlist.id;
                            let n = qobuz_ids.len();
                            if !qobuz_ids.is_empty() {
                                if let Err(e) = runtime
                                    .core()
                                    .add_tracks_to_playlist(pid, &qobuz_ids)
                                    .await
                                {
                                    log::error!(
                                        "[qbz-slint] create-and-add: add failed: {e}"
                                    );
                                }
                                // reco: log the new playlist's Qobuz tracks.
                                let reco_ids = qobuz_ids.clone();
                                tokio::task::spawn_blocking(move || {
                                    crate::reco::log_playlist_add(Some(pid), reco_ids)
                                });
                            }
                            let r2 = runtime.clone();
                            let h2 = handle2.clone();
                            let weak2 = weak.clone();
                            let nm2 = nm.clone();
                            let _ = weak.upgrade_in_event_loop(move |w| {
                                let st = w.global::<PlaylistPickerState>();
                                st.set_creating(false);
                                st.set_creating_open(false);
                                st.set_create_name("".into());
                                // Stays open — see the offline branch above.
                                toast_added_tracks(&weak2, n, nm2);
                                load_sidebar_playlists(r2.clone(), weak2.clone(), &h2);
                                h2.spawn(async move {
                                    let playlists = playlist_picker::load(&r2).await;
                                    let _ = weak2
                                        .upgrade_in_event_loop(move |w| playlist_picker::apply(&w, playlists));
                                });
                            });
                        }
                        Err(e) => {
                            log::error!("[qbz-slint] create-and-add: create failed: {e}");
                            let _ = weak.upgrade_in_event_loop(|w| {
                                w.global::<PlaylistPickerState>().set_creating(false);
                            });
                        }
                    }
                });
            });
    }
}
