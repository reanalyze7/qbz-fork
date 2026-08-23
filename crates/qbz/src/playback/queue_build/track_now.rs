//! Play a single track immediately as a one-track queue.

use super::super::engine::after_track_change;
use super::super::queue_context::make_queue_track;
use super::super::state::refresh_sidebar;
use super::super::Runtime;
use crate::AppWindow;

pub fn play_track_now(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    track_id: u64,
) {
    handle.spawn(async move {
        let track = match runtime.core().get_track(track_id).await {
            Ok(track) => track,
            Err(e) => {
                log::error!("[qbz-slint] playback: get_track {track_id} failed: {e}");
                return;
            }
        };

        let (album_id, album_title, album_artwork) = match track.album.as_ref() {
            Some(album) => (
                album.id.clone(),
                album.title.clone(),
                album.image.best().cloned().unwrap_or_default(),
            ),
            None => (String::new(), String::new(), String::new()),
        };
        let album_artist = track.performer.as_ref().map(|p| p.name.clone()).unwrap_or_default();

        let queue_track =
            make_queue_track(&track, &album_id, &album_title, &album_artist, &album_artwork, None);

        runtime.core().set_queue(vec![queue_track], Some(0)).await;
        after_track_change(&runtime, &weak, track_id).await;
        refresh_sidebar(true);
    });
}
