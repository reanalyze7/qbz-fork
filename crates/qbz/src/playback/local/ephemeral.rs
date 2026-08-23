//! Ephemeral (drag-and-drop / scratch folder) playback entry points.

use super::album::play_local_tracks_now;
use super::super::Runtime;
use crate::AppWindow;

/// Play the whole ephemeral folder (every album, scan order). The in-memory
/// snapshot becomes the queue; playback routes through the shared local-file
/// seam (the synthetic ids resolve via `crate::ephemeral`).
pub fn play_ephemeral_all(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
) {
    handle.spawn(async move {
        let tracks = crate::ephemeral::tracks_snapshot();
        play_local_tracks_now(&runtime, &weak, tracks, 0).await;
    });
}

/// Play one ephemeral album (its tracks become the queue, in scan order).
pub fn play_ephemeral_album(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    group_key: String,
) {
    handle.spawn(async move {
        let tracks = crate::ephemeral::album_tracks(&group_key);
        play_local_tracks_now(&runtime, &weak, tracks, 0).await;
    });
}

/// Play one ephemeral track — its album group becomes the queue, starting at
/// the clicked track (mirrors Tauri's `playEphemeralTrack`).
pub fn play_ephemeral_track(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    track_id: i64,
) {
    handle.spawn(async move {
        let Some(track) = crate::ephemeral::get_track(track_id) else {
            return;
        };
        let key = crate::ephemeral::ephemeral_album_key(&track);
        let tracks = crate::ephemeral::album_tracks(&key);
        let start = tracks.iter().position(|t| t.id == track_id).unwrap_or(0);
        play_local_tracks_now(&runtime, &weak, tracks, start).await;
    });
}

/// Replace the queue with an ephemeral selection identified by intent.
pub fn ephemeral_play(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    kind: String,
    arg: String,
) {
    match kind.as_str() {
        "all" => play_ephemeral_all(runtime, weak, handle),
        "album" => play_ephemeral_album(runtime, weak, handle, arg),
        "track" => {
            if let Ok(id) = arg.parse::<i64>() {
                play_ephemeral_track(runtime, weak, handle, id);
            }
        }
        _ => {}
    }
}
