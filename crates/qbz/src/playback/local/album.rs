//! Local-library album playback + the ephemeral-session wipe helper.
use slint::ComponentHandle;

use super::super::engine::after_track_change;
use super::super::loading::clear_loading;
use super::super::state::refresh_sidebar;
use super::super::Runtime;
use super::queue_track::{fill_missing_covers, local_queue_track};
use qbz_models::QueueTrack;

/// Set a Local Library queue and start playback at `start`. Source-aware
/// `play_audible` routes each track (local file vs offline vs Qobuz) and
/// auto-advance flows through the same path, so a mixed-source album/list
/// plays through. UI-thread async step.
pub(in super::super) async fn play_local_tracks_now(
    runtime: &Runtime,
    weak: &slint::Weak<crate::AppWindow>,
    tracks: Vec<qbz_library::LocalTrack>,
    start: usize,
) {
    if tracks.is_empty() {
        return;
    }
    let queue: Vec<QueueTrack> = tracks.iter().map(local_queue_track).collect();
    let start = start.min(queue.len() - 1);
    let play_id = queue[start].id;
    runtime.core().set_queue(queue, Some(start)).await;
    after_track_change(runtime, weak, play_id).await;
    // Push the new queue onto the sidebar model — without this the Queue
    // panel kept showing the previous queue until it was reopened or its tab
    // toggled. The sibling play paths (play_local_album /
    // play_local_folder_tracks_from / the Qobuz play-all paths) already do
    // this; this shared helper backs all
    // five Local Library entry points, so it was the one path that omitted it.
    refresh_sidebar(true);
}

/// Play a local/offline album (metadata-grouped): the whole album becomes the
/// queue and auto-advances. `album_id` is the metadata group key.
pub fn play_local_album(
    runtime: Runtime,
    weak: slint::Weak<crate::AppWindow>,
    handle: tokio::runtime::Handle,
    album_id: String,
    start_track_id: Option<i64>,
) {
    handle.spawn(async move {
        let tracks = tokio::task::spawn_blocking(move || {
            let mut tracks = crate::local_library::fetch_album_tracks_blocking(&album_id);
            fill_missing_covers(&mut tracks);
            tracks
        })
        .await
        .unwrap_or_default();
        // Start at the requested track (a row click in the album detail) or
        // the top (play-all).
        let start = match start_track_id {
            Some(tid) => tracks.iter().position(|t| t.id == tid).unwrap_or(0),
            None => 0,
        };
        play_local_tracks_now(&runtime, &weak, tracks, start).await;
    });
}

/// If the track currently playing is from an ephemeral folder, stop it and
/// clear the queue + now-playing chrome. Mirrors Tauri's
/// `wipeEphemeralPlaybackArtifacts`: called when the ephemeral session is
/// cleared or replaced, so a stale ephemeral track (whose synthetic id will be
/// reused by the next session) can't linger in the bar or false-highlight a row
/// in the newly-loaded folder.
pub async fn wipe_ephemeral_if_playing(runtime: &Runtime, weak: &slint::Weak<crate::AppWindow>) {
    let is_eph = runtime
        .core()
        .current_track()
        .await
        .map(|t| crate::ephemeral::is_ephemeral_id(t.id as i64))
        .unwrap_or(false);
    if !is_eph {
        return;
    }
    let _ = runtime.core().stop();
    runtime.core().clear_queue(false).await;
    clear_loading(weak, 0);
    let _ = weak.upgrade_in_event_loop(|w| {
        w.global::<crate::NowPlayingState>().set_has_track(false);
    });
    refresh_sidebar(true);
}
