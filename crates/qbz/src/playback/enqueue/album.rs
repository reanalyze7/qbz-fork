//! Album enqueue commands: append, shuffle-play, and play-next.

use super::super::queue_build::play_tracks;
use super::super::queue_context::make_queue_track;
use super::super::quality::album_card_meta;
use super::super::recent_blacklist::track_is_blacklisted_full;
use super::super::state::refresh_sidebar;
use super::super::Runtime;
use crate::AppWindow;
use qbz_models::QueueTrack;

/// Enqueue an album's tracks at the end of the current queue.
pub fn enqueue_album(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    album_id: String,
) {
    handle.spawn(async move {
        let album = match runtime.core().get_album(&album_id).await {
            Ok(album) => album,
            Err(e) => {
                log::error!("[qbz-slint] playback: enqueue get_album {album_id} failed: {e}");
                return;
            }
        };
        let album_title = album.title.clone();
        let album_artist = album.artist.name.clone();
        let album_artwork = album.image.best().cloned().unwrap_or_default();
        crate::recently::remember_album_meta(&album.id, album_card_meta(&album));
        // Drop blacklisted tracks (composer-aware, album-primary fallback)
        // before enqueueing — same predicate as album play-all (D-FIX-b).
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
        runtime.core().add_tracks(tracks).await;
        refresh_sidebar(false);
        crate::toast::success_weak(&weak, qbz_i18n::t("Added to queue"));
    });
}

/// Play an album with its tracks in a fresh random order (the header Shuffle
/// button). Fetches the album, shuffles the raw track list with the same
/// SystemTime-seeded xorshift Fisher-Yates the playlist shuffle uses (no `rand`
/// dependency), then plays from the top via the shared `play_tracks`.
pub fn play_album_shuffled(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    album_id: String,
) {
    let play_handle = handle.clone();
    handle.spawn(async move {
        let album = match runtime.core().get_album(&album_id).await {
            Ok(album) => album,
            Err(e) => {
                log::error!("[qbz-slint] playback: shuffle get_album {album_id} failed: {e}");
                return;
            }
        };
        // D-FEAT: capture the album's primary artist BEFORE moving `tracks`,
        // so the shuffle path applies the SAME album-primary fallback as
        // play-all (fetch_album_for_play). Without it a performer-less album
        // track on a blacklisted artist's album would survive shuffle but be
        // dropped by play-all — an asymmetry on the same album.
        let album_primary = Some(album.artist.id);
        let mut tracks: Vec<qbz_models::Track> =
            album.tracks.map(|container| container.items).unwrap_or_default();
        tracks.retain(|track| !track_is_blacklisted_full(track, album_primary));
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
        play_tracks(runtime, weak, play_handle, tracks, 0);
    });
}
