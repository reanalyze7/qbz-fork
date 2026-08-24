use crate::*;

/// The "checkbox already checked" half of the picker's `on_pick` toggle
/// (spec PLAYLIST-REDESIGN-SPEC.md §4): removes the pending track(s)/refs
/// from `playlist_id` instead of adding them, mirroring the four
/// target/source branches of the add path in shape. Qobuz-playlist +
/// Qobuz-ids is the one branch that pays for an API round trip
/// (`get_playlist`) — only to resolve `playlist_track_id`s, and only when a
/// removal is actually requested (see `playlist_membership_qobuz.rs`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn toggle_off_playlist_pick(
    runtime: &Arc<AppRuntime<SlintAdapter>>,
    weak: &slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    playlist_id: String,
    target_name: String,
    is_local_mode: bool,
    ids_model: &slint::ModelRc<slint::SharedString>,
    track_id_single: &str,
) {
    use slint::Model;
    let mut refs: Vec<String> =
        (0..ids_model.row_count()).filter_map(|i| ids_model.row_data(i)).map(|s| s.to_string()).collect();
    if refs.is_empty() && !track_id_single.is_empty() {
        refs.push(track_id_single.to_string());
    }
    if refs.is_empty() {
        return;
    }
    let weak2 = weak.clone();
    let tname = target_name;

    if local_playlist::is_local_id(&playlist_id) {
        let target = playlist_id;
        let mark_id = target.clone();
        handle.spawn(async move {
            let removed = tokio::task::spawn_blocking(move || {
                playlist_membership::remove_ids_blocking(&target, &refs, is_local_mode)
            })
            .await
            .unwrap_or(0);
            toast_removed_tracks(&weak2, removed, tname);
            if removed > 0 {
                let _ = weak2
                    .upgrade_in_event_loop(move |w| playlist_picker::mark_row_already_has(&w, &mark_id, false));
            }
        });
        return;
    }
    let Ok(pid) = playlist_id.parse::<u64>() else {
        return;
    };
    if is_local_mode {
        handle.spawn(async move {
            let removed =
                tokio::task::spawn_blocking(move || playlist_membership_qobuz::remove_refs_blocking(pid, &refs))
                    .await
                    .unwrap_or(0);
            toast_removed_tracks(&weak2, removed, tname);
            if removed > 0 {
                let _ = weak2.upgrade_in_event_loop(move |w| {
                    playlist_picker::mark_row_already_has(&w, &pid.to_string(), false);
                });
            }
        });
        return;
    }
    let runtime = runtime.clone();
    handle.spawn(async move {
        let want: std::collections::HashSet<u64> = refs.iter().filter_map(|s| s.parse::<u64>().ok()).collect();
        if want.is_empty() {
            return;
        }
        match runtime.core().get_playlist(pid).await {
            Ok(playlist) => {
                let ptids: Vec<u64> = playlist
                    .tracks
                    .map(|c| c.items)
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|t| want.contains(&t.id))
                    .filter_map(|t| t.playlist_track_id)
                    .collect();
                if ptids.is_empty() {
                    return;
                }
                let n = ptids.len();
                if let Err(e) = runtime.core().remove_tracks_from_playlist(pid, &ptids).await {
                    log::error!("[qbz-slint] picker remove failed: {e}");
                } else {
                    toast_removed_tracks(&weak2, n, tname);
                    let _ = weak2.upgrade_in_event_loop(move |w| {
                        playlist_picker::mark_row_already_has(&w, &pid.to_string(), false);
                    });
                }
            }
            Err(e) => log::error!("[qbz-slint] picker remove: get_playlist failed: {e}"),
        }
    });
}

