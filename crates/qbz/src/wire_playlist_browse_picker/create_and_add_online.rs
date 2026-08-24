use crate::*;

// "Create new playlist" -> create-and-add, ONLINE branch (Qobuz playlist,
// then add the carried tracks). Split out of the single `on_create_and_add`
// callback (wire_playlist_browse_picker_part5, part5.rs) to stay under the
// 130-line file cap.
pub(crate) fn spawn_create_and_add_online(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    qobuz_ids: Vec<u64>,
    nm: String,
) {
    let handle2 = handle.clone();
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
}
