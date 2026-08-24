use crate::*;

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

                if offline_now {
                    // D8: offline ⇒ LOCAL playlist (library.db), never the
                    // retired pending-playlist engine. Mirrors the create
                    // modal's offline branch.
                    spawn_create_and_add_offline(
                        runtime, weak, handle.clone(), is_local, refs, qobuz_ids, nm,
                    );
                    return;
                }

                // Online ⇒ Qobuz playlist, then add the carried tracks.
                spawn_create_and_add_online(runtime, weak, handle.clone(), qobuz_ids, nm);
            });
    }
}
