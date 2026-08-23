//! Album context playback: fetch + play-all / play-from-clicked-track.

use super::super::engine::after_track_change;
use super::super::queue_context::{make_queue_track, stamp_queue_context};
use super::super::quality::album_card_meta;
use super::super::recent_blacklist::track_is_blacklisted_full;
use super::super::state::refresh_sidebar;
use super::super::Runtime;
use crate::AppWindow;
use qbz_models::QueueTrack;

/// Play `album_id` from `start_index`: fetch the album, build the queue,
/// hand it to the core, and start audio on the start track.
/// Fetch an album and build its play queue (genre/quality meta cached for
/// the Recently Played card). Shared by `play_album` (start at a positional
/// index) and `play_album_from` (start at a clicked track id). Returns None
/// and toasts on failure / an empty album.
pub(super) async fn fetch_album_for_play(
    runtime: &Runtime,
    weak: &slint::Weak<AppWindow>,
    album_id: &str,
) -> Option<Vec<QueueTrack>> {
    let album = match runtime.core().get_album(album_id).await {
        Ok(album) => album,
        Err(e) => {
            log::error!("[qbz-slint] playback: get_album {album_id} failed: {e}");
            crate::toast::error_weak(weak, qbz_i18n::t("Couldn't load this album"));
            return None;
        }
    };

    let album_title = album.title.clone();
    let album_artist = album.artist.name.clone();
    let album_artwork = album.image.best().cloned().unwrap_or_default();
    // Album's primary artist id — the fallback blacklist key for tracks whose
    // own performer id is missing (D-FEAT album rule: track.artist ?? album).
    let album_primary = Some(album.artist.id);
    // Cache the album's genre / release date / quality so the Recently
    // Played card the play records carries them (no extra fetch).
    crate::recently::remember_album_meta(&album.id, album_card_meta(&album));

    let raw_tracks = album
        .tracks
        .as_ref()
        .map(|container| container.items.as_slice())
        .unwrap_or_default();

    // Genuinely empty album → keep the existing "no playable tracks" toast.
    if raw_tracks.is_empty() {
        log::warn!("[qbz-slint] playback: album {album_id} has no tracks");
        crate::toast::error_weak(weak, qbz_i18n::t("This album has no playable tracks"));
        return None;
    }

    // D-FIX-b: the Tauri `buildAlbumQueueTracks` did NOT filter, so playing an
    // album where a blacklisted artist is FEATURED still queued that track.
    // Filter the raw catalog tracks here (composer-aware, album-primary
    // fallback) BEFORE mapping to QueueTrack so play-all / play-from / shuffle
    // all skip blacklisted (performer OR composer OR featured) tracks.
    let tracks: Vec<QueueTrack> = raw_tracks
        .iter()
        .filter(|track| !track_is_blacklisted_full(track, album_primary))
        .map(|track| {
            make_queue_track(track, &album.id, &album_title, &album_artist, &album_artwork, album.version.as_deref())
        })
        .collect();

    if tracks.is_empty() {
        // Every track was blacklisted → silent early-return (no toast), Tauri
        // 0-playable parity for the album builders.
        log::warn!(
            "[qbz-slint] playback: album {album_id} fully filtered by blacklist; nothing to play"
        );
        return None;
    }
    Some(tracks)
}

pub fn play_album(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    album_id: String,
    start_index: usize,
) {
    handle.spawn(async move {
        let Some(mut tracks) = fetch_album_for_play(&runtime, &weak, &album_id).await else {
            return;
        };
        stamp_queue_context(&mut tracks, "album", &album_id);
        let start = start_index.min(tracks.len() - 1);
        let start_track_id = tracks[start].id;
        runtime.core().set_queue(tracks, Some(start)).await;
        after_track_change(&runtime, &weak, start_track_id).await;
        refresh_sidebar(true);
    });
}

/// Play an album starting at the clicked track id (queues the tracks that
/// follow). `visible_ids` is the album view's VISIBLE row order — the queue
/// is reordered/filtered to match it, so the album track-search filter is
/// respected. Anchoring on the id keeps the start correct regardless.
pub fn play_album_from(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    album_id: String,
    visible_ids: Vec<String>,
    clicked_id: String,
) {
    handle.spawn(async move {
        let Some(tracks) = fetch_album_for_play(&runtime, &weak, &album_id).await else {
            return;
        };
        let mut tracks = super::super::queue_build::reorder_queue_by_visible(tracks, &visible_ids);
        stamp_queue_context(&mut tracks, "album", &album_id);
        let start = tracks
            .iter()
            .position(|t| t.id.to_string() == clicked_id)
            .unwrap_or(0);
        let start_track_id = tracks[start].id;
        runtime.core().set_queue(tracks, Some(start)).await;
        after_track_change(&runtime, &weak, start_track_id).await;
        refresh_sidebar(true);
    });
}
