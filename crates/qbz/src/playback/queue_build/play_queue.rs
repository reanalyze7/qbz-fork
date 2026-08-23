//! Handing a prebuilt queue to the core, and the flat catalog-`Track` list
//! entry points (radio / mix / shuffle) that build one first.

use super::super::engine::after_track_change;
use super::super::queue_context::{make_queue_track, stamp_queue_context};
use super::super::recent_blacklist::track_is_blacklisted_full;
use super::super::state::refresh_sidebar;
use super::super::Runtime;
use crate::AppWindow;
use qbz_models::QueueTrack;

/// Hand a prebuilt `QueueTrack` queue to the core and start at `start`.
/// Callers guard against an empty queue.
pub(in super::super) fn play_queue(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    queue: Vec<QueueTrack>,
    start: usize,
) {
    let start = start.min(queue.len() - 1);
    let first_id = queue[start].id;
    handle.spawn(async move {
        runtime.core().set_queue(queue, Some(start)).await;
        after_track_change(&runtime, &weak, first_id).await;
        refresh_sidebar(true);
    });
}

/// Build a queue from a list of catalog tracks (each carrying its own
/// album) and start playback at `start_index`. Shared by radio
/// (start 0) and the mix views (start at the clicked track).
pub fn play_tracks(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    tracks: Vec<qbz_models::Track>,
    start_index: usize,
) -> bool {
    play_tracks_ctx(runtime, weak, handle, tracks, start_index, None)
}

/// Like [`play_tracks`] but stamps every built queue track with the container
/// it was launched FROM (`Some((kind, id))`) so the now-playing "playing from"
/// button resolves to the right source per track. Pass `None` for flat lists
/// with no container origin (radio / mix / favorites) — those fall back to each
/// track's own album.
pub fn play_tracks_ctx(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    tracks: Vec<qbz_models::Track>,
    start_index: usize,
    context: Option<(String, String)>,
) -> bool {
    // Drop blacklisted tracks (performer OR composer — D-FEAT) before building
    // the queue. Shared sink for radio results, the mix views, and album
    // shuffle, so this single filter covers all three. No album-primary
    // fallback here (these are flat track lists, not an album context).
    let mut queue: Vec<QueueTrack> = tracks
        .iter()
        .filter(|track| !track_is_blacklisted_full(track, None))
        .map(|track| {
            let (album_id, album_title, album_artwork) = track
                .album
                .as_ref()
                .map(|a| (a.id.clone(), a.title.clone(), a.image.best().cloned().unwrap_or_default()))
                .unwrap_or_default();
            let album_artist = track.performer.as_ref().map(|p| p.name.clone()).unwrap_or_default();
            make_queue_track(track, &album_id, &album_title, &album_artist, &album_artwork, None)
        })
        .collect();
    // Stamp the container origin onto every track so the "playing from" button
    // is correct for whichever one is current (republished per change).
    if let Some((kind, id)) = &context {
        stamp_queue_context(&mut queue, kind, id);
    }
    if queue.is_empty() {
        // Either nothing was passed, or every track was blacklisted. Silent
        // early-return (the caller logs); radio callers surface their existing
        // "returned no tracks" warning, matching Tauri's empty->error path.
        return false;
    }
    let start = start_index.min(queue.len() - 1);
    let first_id = queue[start].id;
    handle.spawn(async move {
        runtime.core().set_queue(queue, Some(start)).await;
        after_track_change(&runtime, &weak, first_id).await;
        refresh_sidebar(true);
    });
    true
}
