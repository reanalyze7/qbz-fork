use crate::*;

pub(crate) fn wire_playlist_browse_picker_part6(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {

    // Picker client-side filter — recompute each row's `filter-rank`
    // (case-insensitive substring; Slint 1.16 has no string `.contains`, so
    // the match runs here). Rank = match ordinal among the filtered rows,
    // -1 = filtered out; the total lands in `filter-matches`. Pure frontend,
    // no backend call.
    {
        let weak = window.as_weak();
        window
            .global::<PlaylistPickerActions>()
            .on_filter_changed(move |query| {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                use slint::Model;
                let needle = query.to_lowercase();
                let model = w.global::<PlaylistPickerState>().get_playlists();
                let mut rank: i32 = 0;
                for i in 0..model.row_count() {
                    if let Some(mut item) = model.row_data(i) {
                        let matches = needle.is_empty()
                            || item.name.to_lowercase().contains(&needle);
                        let new_rank = if matches { rank } else { -1 };
                        if matches {
                            rank += 1;
                        }
                        if item.filter_rank != new_rank {
                            item.filter_rank = new_rank;
                            model.set_row_data(i, item);
                        }
                    }
                }
                w.global::<PlaylistPickerState>().set_filter_matches(rank);
            });
    }

    // Duplicate-confirm sub-modal handlers. The pending context lives in
    // DUP_CONFIRM_STASH (set by the picker's Qobuz→Qobuz branch). Each handler
    // reads it, performs the chosen add, toasts, then closes + clears.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<DuplicateConfirmActions>()
            .on_add_all(move || {
                let Some(stash) = DUP_CONFIRM_STASH.with(|c| c.borrow_mut().take()) else {
                    return;
                };
                let (pid, all_ids, _dup_ids, name) = stash;
                if let Some(w) = weak.upgrade() {
                    w.global::<DuplicateConfirmState>().set_busy(true);
                }
                let runtime = runtime.clone();
                let weak = weak.clone();
                handle.spawn(async move {
                    let n = all_ids.len();
                    if let Err(e) = runtime.core().add_tracks_to_playlist(pid, &all_ids).await
                    {
                        log::error!("[qbz-slint] dup add-all failed: {e}");
                    } else {
                        // reco: log the full requested Qobuz ids (add-all).
                        let reco_ids = all_ids.clone();
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
}
