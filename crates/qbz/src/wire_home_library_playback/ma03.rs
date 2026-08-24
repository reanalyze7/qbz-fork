use crate::*;

// One batch of `on_media_action` match arms, split out of the original
// single 2087-line callback (crates/qbz/src/main.rs refactor) to stay
// under the 130-line file cap. Called unconditionally in original arm
// order from `dispatch_media_action` (ma_dispatch.rs); each batch's
// `match` only fires for its own (kind, action) patterns, no-op otherwise.
pub(crate) fn ma_batch03(
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
                ("track", "play") => {
                    // Universal per-row play: queue the current view's VISIBLE
                    // tracklist starting at the clicked track (see
                    // playback::play_track_in_context). Every tracklist surface
                    // routes here — album, playlist, favorites, label, mix,
                    // artist, search.
                    if let Some(w) = weak.upgrade() {
                        playback::play_track_in_context(
                            &w,
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            &id,
                        );
                    }
                }
                ("album", "queue") => playback::enqueue_album(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    id,
                ),
                ("track", "queue") => {
                    // SOURCE-TYPED routing first (spec §3.2, mirrors the
                    // add-to-playlist arm): on a snapshot-backed playlist
                    // detail a local row's id is a library row id — the
                    // catalog path below would mis-resolve it (wrong-track
                    // hazard / silent failure). The merged snapshot carries
                    // the ready, source-aware QueueTrack; enqueue it directly.
                    if let Some(w) = weak.upgrade() {
                        if snapshot_detail_open(&w) {
                            if let Some(qt) = local_playlist::queue_track_for_row(&id) {
                                if matches!(qt.source.as_deref(), Some("local")) {
                                    playback::enqueue_queue_tracks(
                                        runtime.clone(),
                                        weak.clone(),
                                        handle.clone(),
                                        vec![qt],
                                        false,
                                    );
                                    return;
                                }
                            }
                        }
                    }
                    // Qobuz rows (incl. offline copies with real catalog
                    // ids): the existing path — single-track
                    // admission + fresh fetch.
                    if let Ok(track_id) = id.parse::<u64>() {
                        playback::enqueue_track(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            track_id,
                        );
                    }
                }
                ("album", "play-next") => playback::enqueue_album_next(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    id,
                ),
                ("album", "shuffle") => playback::play_album_shuffled(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    id,
                ),
                ("album", "edit") => {
                    // Open the local-album tag editor (group_key == directory_path
                    // for folder-grouped local albums).
                    tag_editor::open_tag_editor(weak.clone(), handle.clone(), id.clone(), id);
                }
        _ => {}
    }
}
