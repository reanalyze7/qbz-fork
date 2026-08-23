//! Batch enqueue of already-built, SOURCE-AWARE `QueueTrack`s, plus the
//! LocalLibrary-row batch enqueue.

use super::super::local::queue_track::local_queue_track;
use super::super::recent_blacklist::filter_blacklisted_queue;
use super::super::state::refresh_sidebar;
use super::super::Runtime;
use crate::AppWindow;
use qbz_models::QueueTrack;

/// Append (or insert-next) a batch of already-built, SOURCE-AWARE
/// QueueTracks — the playlist detail's per-row / bulk Play next + Add to
/// queue route their snapshot rows here (local/cached rows keep their
/// source, so `play_audible` resolves each through its own path). QConnect
/// CONTROLLER mode rides the same batch admission as `enqueue_playlist`:
/// all-or-nothing — a non-castable (local) row refuses the whole batch
/// with a toast while a peer owns playback, exactly like the other
/// source-typed batch paths.
pub fn enqueue_queue_tracks(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    tracks: Vec<QueueTrack>,
    next: bool,
) {
    if tracks.is_empty() {
        return;
    }
    // Drop blacklisted Qobuz rows (performer; local/cached rows kept by the
    // source guard). Silent early-return when nothing playable remains.
    let tracks = filter_blacklisted_queue(tracks);
    if tracks.is_empty() {
        return;
    }
    handle.spawn(async move {
        if next {
            // Reverse so the inserted block keeps the selection's order.
            for track in tracks.into_iter().rev() {
                runtime.core().add_track_next(track).await;
            }
        } else {
            runtime.core().add_tracks(tracks).await;
        }
        refresh_sidebar(false);
        crate::toast::success_weak(
            &weak,
            if next { qbz_i18n::t("Playing next") } else { qbz_i18n::t("Added to queue") },
        );
    });
}

/// Append (or insert-next) a batch of already-loaded LocalLibrary rows to the
/// queue. Mirrors `enqueue_tracks` but for `LocalTrack`: `local_queue_track`
/// builds source-aware QueueTracks (is_local=true; "local"/"qobuz_download")
/// so `play_audible` routes user files through the protected `play_data` seam
/// and offline copies through `play_track_resolved`. Reversed for "play next"
/// to preserve selection order.
pub fn enqueue_local_tracks(
    runtime: Runtime,
    handle: tokio::runtime::Handle,
    tracks: Vec<qbz_library::LocalTrack>,
    next: bool,
) {
    if tracks.is_empty() {
        return;
    }
    handle.spawn(async move {
        let ordered: Vec<qbz_library::LocalTrack> = if next {
            tracks.into_iter().rev().collect()
        } else {
            tracks
        };
        for track in &ordered {
            let qt = local_queue_track(track);
            if next {
                runtime.core().add_track_next(qt).await;
            } else {
                runtime.core().add_track(qt).await;
            }
        }
        refresh_sidebar(false);
    });
}
