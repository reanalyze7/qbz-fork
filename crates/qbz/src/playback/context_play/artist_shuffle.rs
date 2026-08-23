//! Shuffle-play for an artist's or label's Popular tracks.

use super::artist_fetch::fetch_artist_top_for_play;
use super::super::engine::after_track_change;
use super::super::queue_context::stamp_queue_context;
use super::super::state::refresh_sidebar;
use super::super::Runtime;
use crate::AppWindow;
use qbz_models::Track;

/// Shuffle-play ALL of the artist's Popular tracks (section "more" menu).
/// Re-fetches, xorshift-shuffles (same seedless mix as `play_album_shuffled`),
/// and replaces the queue.
pub fn play_artist_top_shuffled(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    artist_id: String,
) {
    handle.spawn(async move {
        let Some(mut tracks) = fetch_artist_top_for_play(&runtime, &weak, &artist_id).await else {
            return;
        };
        if tracks.is_empty() {
            return;
        }
        let mut seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(1)
            | 1;
        for i in (1..tracks.len()).rev() {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            let j = (seed % (i as u64 + 1)) as usize;
            tracks.swap(i, j);
        }
        stamp_queue_context(&mut tracks, "artist", &artist_id);
        let start_track_id = tracks[0].id;
        runtime.core().set_queue(tracks, Some(0)).await;
        after_track_change(&runtime, &weak, start_track_id).await;
        refresh_sidebar(true);
    });
}

/// Shuffle-play ALL of the label's Popular tracks (label header shuffle).
/// Tracks come from the label page's cached list (no re-fetch needed);
/// xorshift-shuffles in place (same seedless mix as `play_album_shuffled`)
/// and hands off to `play_tracks_ctx` with the label context stamped.
pub fn play_label_top_shuffled(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    tracks: Vec<Track>,
    label_id: String,
) -> bool {
    let mut tracks = tracks;
    if tracks.is_empty() {
        return false;
    }
    let mut seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(1)
        | 1;
    for i in (1..tracks.len()).rev() {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        let j = (seed % (i as u64 + 1)) as usize;
        tracks.swap(i, j);
    }
    super::super::queue_build::play_tracks_ctx(
        runtime,
        weak,
        handle,
        tracks,
        0,
        Some(("label".to_string(), label_id)),
    )
}
