//! Single-track enqueue / play-next.

use super::super::queue_context::make_queue_track;
use super::super::state::refresh_sidebar;
use super::super::Runtime;
use crate::AppWindow;

/// Enqueue a single track at the end of the current queue.
pub fn enqueue_track(runtime: Runtime, weak: slint::Weak<AppWindow>, handle: tokio::runtime::Handle, track_id: u64) {
    handle.spawn(async move {
        let track = match runtime.core().get_track(track_id).await {
            Ok(track) => track,
            Err(e) => {
                log::error!("[qbz-slint] playback: enqueue get_track {track_id} failed: {e}");
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
        runtime.core().add_track(queue_track).await;
        refresh_sidebar(false);
        crate::toast::success_weak(&weak, qbz_i18n::t("Added to queue"));
    });
}

/// Insert a single track immediately after the current track ("Play next").
pub fn play_track_next(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    track_id: u64,
) {
    handle.spawn(async move {
        let track = match runtime.core().get_track(track_id).await {
            Ok(track) => track,
            Err(e) => {
                log::error!("[qbz-slint] playback: play-next get_track {track_id} failed: {e}");
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
        runtime.core().add_track_next(queue_track).await;
        refresh_sidebar(false);
        crate::toast::success_weak(&weak, qbz_i18n::t("Playing next"));
    });
}
