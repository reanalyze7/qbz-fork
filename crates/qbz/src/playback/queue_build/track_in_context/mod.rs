//! Per-row "play this track" for EVERY tracklist surface: `play_track_in_context`
//! and its per-view branches (each split into its own file to stay under the
//! line budget).
use slint::ComponentHandle;

mod other_views;
mod playlist;
mod search;

use super::track_now::play_track_now;
use super::super::Runtime;
use crate::{AppWindow, ContentView, NavState};

/// Per-row "play this track" for EVERY tracklist surface. Builds the queue
/// from the CURRENT view's VISIBLE list and starts at the clicked track, so
/// the tracks that visually follow it play next — regardless of context
/// (playlist custom sort, album, favorites filter, artist top tracks, ...).
///
/// This is the single entry point for clicking/double-clicking a track row.
/// It replaces a scatter of per-view paths that each got it wrong: the album
/// row played a lone track (no queue), and the playlist/mix rows always
/// started at track 1 (the clicked id was read from the wrong media-action
/// slot). Do NOT reintroduce per-view play arms — route everything here.
pub fn play_track_in_context(
    window: &AppWindow,
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    clicked_id: &str,
) {
    let view = window.global::<NavState>().get_view();
    // Playback context is now stamped PER-TRACK on the enqueued queue (see
    // stamp_queue_context) and republished every track change in
    // refresh_now_playing_meta. The playlist/label branches below stamp their
    // container; favorites/mix/search/single-track leave the tracks unstamped so
    // the song-card layers button falls back to each track's own album. No
    // global clear needed — a fresh set_queue replaces the whole queue, and an
    // unstamped current track derives the album fallback authoritatively.
    let handled = match view {
        ContentView::Playlist => playlist::handle(window, &runtime, &weak, &handle, clicked_id),
        ContentView::Favorites => other_views::handle_favorites(window, &runtime, &weak, &handle, clicked_id),
        ContentView::Label => other_views::handle_label(window, &runtime, &weak, &handle, clicked_id),
        ContentView::Mix => other_views::handle_mix(&runtime, &weak, &handle, clicked_id),
        ContentView::Search => search::handle(window, &runtime, &weak, &handle, clicked_id),
        ContentView::Album => other_views::handle_album(window, &runtime, &weak, &handle, clicked_id),
        ContentView::Artist => other_views::handle_artist(window, &runtime, &weak, &handle, clicked_id),
        _ => false,
    };
    if handled {
        return;
    }
    // No resolvable list context (Home, Discover, ...): play just the track.
    if let Ok(tid) = clicked_id.parse::<u64>() {
        play_track_now(runtime, weak, handle, tid);
    }
}
