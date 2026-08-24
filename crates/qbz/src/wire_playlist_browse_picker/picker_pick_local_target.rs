use crate::*;

// `on_pick` ADD branch when the target playlist id is a LOCAL playlist
// ("local:<uuid>" — writes go to the library.db repo, works offline, D7
// routing). Handles both local-mode refs and Qobuz-id sources. Split out
// of the single `on_pick` callback (wire_playlist_browse_picker_part3,
// part3.rs) to stay under the 130-line file cap.
pub(crate) fn picker_pick_local_target(
    runtime: &Arc<AppRuntime<SlintAdapter>>,
    weak: &slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    playlist_id: &str,
    is_local: bool,
    ids_model: &slint::ModelRc<slint::SharedString>,
    track_id_single: &str,
    target_name: String,
) {
    let _ = runtime;
                if local_playlist::is_local_id(playlist_id) {
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
}
