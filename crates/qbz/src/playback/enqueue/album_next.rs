//! Insert an album's tracks after the current track ("Play next"), split
//! out of `album.rs` to keep both files under the line budget.

use super::super::queue_context::make_queue_track;
use super::super::quality::album_card_meta;
use super::super::recent_blacklist::track_is_blacklisted_full;
use super::super::state::refresh_sidebar;
use super::super::Runtime;
use crate::AppWindow;
use qbz_models::QueueTrack;

/// Insert an album's tracks immediately after the current track ("Play next").
///
/// The core's `add_track_next` inserts a single track after the current index,
/// so the album tracks are inserted in reverse order to land in the right
/// sequence — mirroring Tauri's `v2_add_tracks_to_queue_next`.
pub fn enqueue_album_next(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    album_id: String,
) {
    handle.spawn(async move {
        let album = match runtime.core().get_album(&album_id).await {
            Ok(album) => album,
            Err(e) => {
                log::error!("[qbz-slint] playback: play-next get_album {album_id} failed: {e}");
                return;
            }
        };
        let album_title = album.title.clone();
        let album_artist = album.artist.name.clone();
        let album_artwork = album.image.best().cloned().unwrap_or_default();
        crate::recently::remember_album_meta(&album.id, album_card_meta(&album));
        // Drop blacklisted tracks (composer-aware, album-primary fallback)
        // before play-next — same predicate as album play-all (D-FIX-b).
        let album_primary = Some(album.artist.id);
        let tracks: Vec<QueueTrack> = album
            .tracks
            .as_ref()
            .map(|container| container.items.as_slice())
            .unwrap_or_default()
            .iter()
            .filter(|track| !track_is_blacklisted_full(track, album_primary))
            .map(|track| {
                make_queue_track(track, &album.id, &album_title, &album_artist, &album_artwork, album.version.as_deref())
            })
            .collect();
        if tracks.is_empty() {
            return;
        }
        // Insert in reverse so the tracks end up in the correct order.
        for track in tracks.into_iter().rev() {
            runtime.core().add_track_next(track).await;
        }
        refresh_sidebar(false);
        crate::toast::success_weak(&weak, qbz_i18n::t("Playing next"));
    });
}
