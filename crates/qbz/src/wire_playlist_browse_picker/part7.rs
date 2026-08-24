use crate::*;
use crate::navigate_album_artist::nav_statics::DUP_CONFIRM_STASH;

pub(crate) fn wire_playlist_browse_picker_part7(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, _image_cache: &artwork::ImageCache, _settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<DuplicateConfirmActions>()
            .on_add_new_only(move || {
                let Some(stash) = DUP_CONFIRM_STASH.with(|c| c.borrow_mut().take()) else {
                    return;
                };
                let (pid, all_ids, dup_ids, name) = stash;
                // reco: keep the FULL requested ids before the dedup consumes
                // them (Tauri logs the full request, not the inserted subset).
                let reco_all = all_ids.clone();
                // Add only the non-duplicate ids (all \ duplicates). If nothing
                // is left, just close.
                let to_add: Vec<u64> =
                    all_ids.into_iter().filter(|id| !dup_ids.contains(id)).collect();
                if to_add.is_empty() {
                    if let Some(w) = weak.upgrade() {
                        w.global::<DuplicateConfirmState>().set_open(false);
                    }
                    return;
                }
                if let Some(w) = weak.upgrade() {
                    w.global::<DuplicateConfirmState>().set_busy(true);
                }
                let runtime = runtime.clone();
                let weak = weak.clone();
                handle.spawn(async move {
                    let n = to_add.len();
                    if let Err(e) = runtime.core().add_tracks_to_playlist(pid, &to_add).await
                    {
                        log::error!("[qbz-slint] dup add-new-only failed: {e}");
                    } else {
                        // reco: log the FULL requested ids (Tauri parity), not
                        // just the non-duplicate subset that was inserted.
                        let reco_ids = reco_all;
                        tokio::task::spawn_blocking(move || {
                            crate::reco::log_playlist_add(Some(pid), reco_ids)
                        });
                        toast_added_tracks(&weak, n, name);
                    }
                    let _ = weak.upgrade_in_event_loop(move |w| {
                        let st = w.global::<DuplicateConfirmState>();
                        st.set_busy(false);
                        st.set_open(false);
                        playlist_picker::mark_row_already_has(&w, &pid.to_string(), true);
                    });
                });
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<DuplicateConfirmActions>()
            .on_cancel(move || {
                DUP_CONFIRM_STASH.with(|c| *c.borrow_mut() = None);
                if let Some(w) = weak.upgrade() {
                    let st = w.global::<DuplicateConfirmState>();
                    st.set_busy(false);
                    st.set_open(false);
                }
            });
    }

    // Track drag onto sidebar playlists (a star QBZ feature).
    {
        let weak = window.as_weak();
        window.global::<DragActions>().on_start(
            move |track_id, title, subtitle, gx, gy| {
                let Some(w) = weak.upgrade() else { return };
                log::info!("[qbz-slint][drag] start gx={gx} gy={gy} (cursor should be here)");
                let tracks = gather_drag_tracks(&w, track_id.as_str());
                let count = tracks.len();
                drag::set_dragged(tracks);
                let ds = w.global::<DragState>();
                ds.set_count(count as i32);
                ds.set_ghost_title(title);
                ds.set_ghost_subtitle(subtitle);
                ds.set_pointer_x(gx);
                ds.set_pointer_y(gy);
                ds.set_over_playlist_id("".into());
                ds.set_active(true);
            },
        );
    }
}
