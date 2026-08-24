use crate::*;

/// Favorites view actions — tab switch (lazy-load), open album / artist,
/// and per-row track actions routed to the media-action "Add to playlist"
/// picker — pick TOGGLES membership (checkbox semantics, spec
/// PLAYLIST-REDESIGN-SPEC.md §4): not-yet-present adds the pending
/// track(s), already-present removes them. Never closes the picker (only
/// close() does — footer "Done" / backdrop); close dismisses.
///
/// The ADD branches are split into `picker_pick_local_target`,
/// `picker_pick_local_refs_onto_qobuz`, and `picker_pick_qobuz_target`
/// (this dir's `picker_pick_*.rs`) to stay under the 130-line file cap.
pub(crate) fn wire_playlist_browse_picker_part3(
    window: &AppWindow,
    app_runtime: &Arc<AppRuntime<SlintAdapter>>,
    tokio_rt: &tokio::runtime::Runtime,
    image_cache: &artwork::ImageCache,
    settings_ctx: &Arc<settings::SettingsCtx>,
) {
    let _ = settings_ctx;
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
                    picker_pick_local_target(
                        &runtime,
                        &weak,
                        &handle,
                        playlist_id.as_str(),
                        is_local,
                        &ids_model,
                        &track_id_single,
                        target_name,
                    );
                    return;
                }

                let Ok(pid) = playlist_id.parse::<u64>() else {
                    return;
                };

                if is_local {
                    picker_pick_local_refs_onto_qobuz(
                        runtime.clone(),
                        weak.clone(),
                        handle.clone(),
                        image_cache.clone(),
                        pid,
                        &ids_model,
                        target_name,
                    );
                    return;
                }

                picker_pick_qobuz_target(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    pid,
                    &ids_model,
                    &track_id_single,
                    target_name,
                );
            });
    }
}
