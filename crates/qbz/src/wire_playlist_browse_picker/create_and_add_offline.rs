use crate::*;

// "Create new playlist" -> create-and-add, OFFLINE branch (D8: LOCAL
// playlist via library.db, never the retired pending-playlist engine).
// Split out of the single `on_create_and_add` callback
// (wire_playlist_browse_picker_part5, part5.rs) to stay under the
// 130-line file cap.
pub(crate) fn spawn_create_and_add_offline(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    is_local: bool,
    refs: Vec<String>,
    qobuz_ids: Vec<u64>,
    nm: String,
) {
    let handle2 = handle.clone();
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
}
