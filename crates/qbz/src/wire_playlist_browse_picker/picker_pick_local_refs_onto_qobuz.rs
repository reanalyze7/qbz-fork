use crate::*;

// `on_pick` ADD branch: local-mode refs (LocalLibrary row ids) onto a QOBUZ
// playlist — row ids attach via the local sidecar (same table the offline
// detail renders). Split out of the single `on_pick` callback
// (wire_playlist_browse_picker_part3, part3.rs) to stay under the
// 130-line file cap.
pub(crate) fn picker_pick_local_refs_onto_qobuz(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    image_cache: artwork::ImageCache,
    pid: u64,
    ids_model: &slint::ModelRc<slint::SharedString>,
    target_name: String,
) {
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
}
