use crate::*;

// One batch of `on_media_action` match arms, split out of the original
// single 2087-line callback (crates/qbz/src/main.rs refactor) to stay
// under the 130-line file cap. Called unconditionally in original arm
// order from `dispatch_media_action` (ma_dispatch.rs); each batch's
// `match` only fires for its own (kind, action) patterns, no-op otherwise.
pub(crate) fn ma_batch09(
    kind: &str,
    id: &str,
    action: &str,
    runtime: &Arc<AppRuntime<SlintAdapter>>,
    weak: &slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: &artwork::ImageCache,
) {
    let runtime = runtime.clone();
    let weak = weak.clone();
    let handle = handle.clone();
    let image_cache = image_cache.clone();
    let id = id.to_string();
    match (kind, action) {
                ("track", "favorite") => {
                    // Offline guard + optimistic toggle + network flip with
                    // rollback — shared with the library-surface favorite
                    // (see toggle_track_favorite).
                    toggle_track_favorite(
                        runtime.clone(),
                        weak.clone(),
                        handle.clone(),
                        id.to_string(),
                    );
                }
                // Offline cache: "download"/"cache" make a track available
                // offline; "uncache" removes the local copy. The row affordance
                // and the context menu both bubble these.
                ("track", "cache") | ("track", "download") => {
                    if let Ok(track_id) = id.parse::<u64>() {
                        offline_cache::cache_track(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            track_id,
                        );
                    }
                }
                ("track", "uncache") => {
                    if let Ok(track_id) = id.parse::<u64>() {
                        offline_cache::remove_cached(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            track_id,
                        );
                    }
                }
                ("track", "recache") => {
                    // "Refresh offline copy" (cached-state menu entry, spec
                    // §3.5): remove + re-download, sequenced.
                    if let Ok(track_id) = id.parse::<u64>() {
                        offline_cache::refresh_cached(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            track_id,
                        );
                    }
                }
                ("track", "remove-from-playlist") => {
                    // Per-row removal on the playlist detail (spec §3.1).
                    // Ownership-gated in the UI (PlaylistState.is-owner —
                    // DELIBERATE: Tauri's available branch renders it
                    // un-gated on followed playlists where the owner-only
                    // API rejects, §1.6.1; we port the intent, not the
                    // hole). One-row ride on the same namespace-split seam
                    // as the bulk removal; the reload re-merges the sidecar.
                    let Some(w) = weak.upgrade() else { return };
                    if w.global::<NavState>().get_view() != ContentView::Playlist {
                        return;
                    }
                    if w.global::<PlaylistState>().get_is_local() {
                        local_playlist::remove_rows_by_ids(
                            &w,
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            image_cache.clone(),
                            vec![id.to_string()],
                        );
                        return;
                    }
                    let pid = w.global::<PlaylistState>().get_id().to_string();
                    let Some(row) = playlist::row_for_id(&id) else {
                        log::warn!("[qbz-slint] remove-from-playlist: row {id} not loaded");
                        return;
                    };
                    if let Ok(pid) = pid.parse::<u64>() {
                        playlist_remove_rows(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            image_cache.clone(),
                            pid,
                            vec![row],
                        );
                    }
                }
                // External-reco Weekly rows (P7): the title-adjacent buttons.
                // `id` carries the section key ("weekly-exploration"/"weekly-jams").
        _ => {}
    }
}
