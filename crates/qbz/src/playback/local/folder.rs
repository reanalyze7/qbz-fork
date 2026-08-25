//! Local-library folder/list playback entry points.
use slint::ComponentHandle;

use super::album::play_local_tracks_now;
use super::queue_track::fill_missing_covers;
use super::super::state::refresh_sidebar;
use super::super::Runtime;
use crate::{AppWindow, NowPlayingState};

/// Play an explicit list of local tracks (already resolved — e.g. one album
/// VERSION), starting at `start`. `shuffle` enables shuffle mode after the
/// queue is set. Used by the dedicated Local album view so it plays the SHOWN
/// version, never a re-merged metadata group.
pub fn play_local_tracks(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    tracks: Vec<qbz_library::LocalTrack>,
    start: usize,
    shuffle: bool,
) {
    if tracks.is_empty() {
        return;
    }
    handle.spawn(async move {
        play_local_tracks_now(&runtime, &weak, tracks, start).await;
        if shuffle {
            // No set_shuffle on core — toggle until it's on.
            let mut on = runtime.core().toggle_shuffle().await;
            if !on {
                on = runtime.core().toggle_shuffle().await;
            }
            let _ = weak.upgrade_in_event_loop(move |w| {
                w.global::<NowPlayingState>().set_shuffle(on);
            });
            // `play_local_tracks_now` already refreshed the sidebar BEFORE this
            // inline toggle, so the UP NEXT list is still in pre-shuffle order —
            // re-pull it now that shuffle reordered the queue. `false` = no fav
            // network pull.
            refresh_sidebar(false);
        }
    });
}

/// Play everything under a folder (recursive), in path order — the whole
/// subtree becomes the queue. Mirrors `play_local_album` but sources the
/// tracks from the folder hierarchy instead of a metadata group.
pub fn play_local_folder_recursive(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    folder_path: String,
) {
    handle.spawn(async move {
        let tracks = tokio::task::spawn_blocking(move || {
            let mut tracks = crate::library_db::with_db(|db| {
                db.list_folder_tracks_recursive(&folder_path, false)
            })
            .unwrap_or_default();
            fill_missing_covers(&mut tracks);
            tracks
        })
        .await
        .unwrap_or_default();
        if tracks.is_empty() {
            return;
        }
        play_local_tracks_now(&runtime, &weak, tracks, 0).await;
    });
}

/// Play a folder's DIRECT tracks (non-recursive) starting at `start_track_id`
/// — the folder's own track list becomes the queue. Used by the tree-mode
/// detail pane when a track row is clicked.
pub fn play_local_folder_tracks_from(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    folder_path: String,
    start_track_id: i64,
) {
    handle.spawn(async move {
        let tracks = tokio::task::spawn_blocking(move || {
            let mut tracks =
                crate::library_db::with_db(|db| db.list_folder_tracks(&folder_path, false))
                    .unwrap_or_default();
            fill_missing_covers(&mut tracks);
            tracks
        })
        .await
        .unwrap_or_default();
        if tracks.is_empty() {
            return;
        }
        let start = tracks
            .iter()
            .position(|t| t.id == start_track_id)
            .unwrap_or(0);
        play_local_tracks_now(&runtime, &weak, tracks, start).await;
    });
}
