use crate::*;
use crate::navigate_album_artist::nav_statics::DUP_CONFIRM_STASH;

// `on_pick` ADD branch: Qobuz tracks -> Qobuz playlist. Runs duplicate
// detection first (Tauri parity: this is the ONLY branch that checks
// dupes) — if any ids are already in the target, stash the context and
// open the confirm sub-modal; otherwise add directly and toast. Split out
// of the single `on_pick` callback (wire_playlist_browse_picker_part3,
// part3.rs) to stay under the 130-line file cap.
pub(crate) fn picker_pick_qobuz_target(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    pid: u64,
    ids_model: &slint::ModelRc<slint::SharedString>,
    track_id_single: &str,
    target_name: String,
) {
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
}
