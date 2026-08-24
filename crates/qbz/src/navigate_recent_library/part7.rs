use crate::*;

/// Namespace-split removal from the ONLINE Qobuz playlist detail (Seam D):
/// Qobuz rows go to the Qobuz API as `playlist_track_id`s (resolved through
/// the loaded detail — fixing the old bulk path that shipped TRACK ids),
/// local rows to `remove_local_track_from_playlist`; then the detail reloads
/// (re-merge).
/// The bulk bar calls this with the selection; the per-row "Remove from
/// playlist" menu entry (follow-up) calls it with a single row.
pub(crate) fn playlist_remove_rows(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    image_cache: artwork::ImageCache,
    pid: u64,
    rows: Vec<playlist::SelectedRow>,
) {
    // Resolve on the UI thread: ptids from the loaded Track cache.
    let split = playlist::split_for_removal(&rows);
    if split.playlist_track_ids.is_empty() && split.local_track_ids.is_empty() {
        log::warn!("[qbz-slint] playlist {pid}: nothing resolvable in the removal selection");
        return;
    }
    handle.clone().spawn(async move {
        let local_ids = split.local_track_ids;
        if !local_ids.is_empty() {
            let _ = tokio::task::spawn_blocking(move || {
                crate::library_db::with_db(|db| {
                    for rid in &local_ids {
                        db.remove_local_track_from_playlist(pid, *rid)?;
                    }
                    Ok(())
                })
            })
            .await;
        }
        if !split.playlist_track_ids.is_empty() {
            if let Err(e) = runtime
                .core()
                .remove_tracks_from_playlist(pid, &split.playlist_track_ids)
                .await
            {
                log::error!("[qbz-slint] remove tracks from playlist failed: {e}");
            }
        }
        // Reload + leave edit mode (the reload re-merges the sidecar).
        let _ = weak.upgrade_in_event_loop(|w| {
            playlist::set_multi_select(&w, false);
        });
        navigate_playlist(
            runtime.clone(),
            weak.clone(),
            &handle,
            image_cache.clone(),
            pid.to_string(),
        );
    });
}

/// True while the OPEN view is a playlist detail whose rows ride the merged
/// queue snapshot (LOCAL detail / offline subset / ONLINE mixed detail) —
/// the guard for consulting snapshot row ids from the universal track arms.
/// Only then may a row id be a library row id; a stale snapshot id could
/// otherwise collide with a genuine Qobuz catalog id from another surface
/// (both are small integers).
pub(crate) fn snapshot_detail_open(w: &AppWindow) -> bool {
    w.global::<NavState>().get_view() == ContentView::Playlist
        && (w.global::<PlaylistState>().get_is_local()
            || w.global::<PlaylistState>().get_offline_subset()
            || playlist::is_mixed())
}

