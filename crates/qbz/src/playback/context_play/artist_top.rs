//! Play the artist's Popular tracks from the top.

use super::artist_fetch::fetch_artist_top_for_play;
use super::super::engine::after_track_change;
use super::super::queue_context::stamp_queue_context;
use super::super::state::refresh_sidebar;
use super::super::Runtime;
use crate::AppWindow;

/// Play the artist's top tracks as a fresh queue, starting at the
/// first track. Wired to the Popular Tracks "play all" CircleAction
/// in ArtistPageView. Re-fetches the artist page so the queue
/// carries the same audio metadata the page row uses.
pub fn play_artist_top_tracks(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    artist_id: String,
) {
    handle.spawn(async move {
        let Some(mut tracks) = fetch_artist_top_for_play(&runtime, &weak, &artist_id).await else {
            return;
        };
        stamp_queue_context(&mut tracks, "artist", &artist_id);
        let start_track_id = tracks[0].id;
        runtime.core().set_queue(tracks, Some(0)).await;
        after_track_change(&runtime, &weak, start_track_id).await;
        refresh_sidebar(true);
    });
}
