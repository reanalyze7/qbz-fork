//! Playlist play-now and enqueue, both fetching fresh so they work from any
//! playlist CARD, not just an open detail view.

use super::super::engine::after_track_change;
use super::super::queue_context::stamp_queue_context;
use super::super::recent_blacklist::filter_blacklisted_queue;
use super::super::state::refresh_sidebar;
use super::super::Runtime;
use crate::AppWindow;
use qbz_models::{QueueTrack, Track};

/// Play a whole playlist (by id) NOW — replace the queue with the playlist's
/// tracks and start at the first one. Fetches the tracks fresh, so it works
/// from any playlist CARD (Discover / Search / Label carousels) without a
/// PlaylistView open, unlike the `play-all` arm (which reads the open detail's
/// PlaylistState). Mirrors `enqueue_playlist`'s fetch + mixed-sidecar interleave
/// but calls `set_queue` instead of `add_tracks`, like `play_album`.
pub fn play_playlist(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    playlist_id: String,
) {
    let Ok(pid) = playlist_id.parse::<u64>() else {
        return;
    };
    handle.spawn(async move {
        let playlist = match runtime.core().get_playlist(pid).await {
            Ok(playlist) => playlist,
            Err(e) => {
                log::error!("[qbz-slint] playback: play get_playlist {pid} failed: {e}");
                return;
            }
        };
        let qobuz_tracks: Vec<Track> = playlist.tracks.map(|c| c.items).unwrap_or_default();
        // Same mixed-playlist merge as `enqueue_playlist`: interleave the
        // local sidecar rows at their stored slots so a card play carries
        // every row WITH its source. Pure-Qobuz playlists read an empty sidecar.
        let qobuz_count = qobuz_tracks.len() as u32;
        let sidecar = tokio::task::spawn_blocking(move || {
            crate::local_playlist::read_sidecar_rows_blocking(pid, qobuz_count)
        })
        .await
        .unwrap_or_default();
        let rows = crate::playlist::interleave_rows(qobuz_tracks, sidecar);
        // Drop blacklisted Qobuz rows (performer; local rows kept by the
        // source guard). Silent early-return when nothing playable remains.
        let mut tracks: Vec<QueueTrack> = filter_blacklisted_queue(
            rows.iter()
                .filter_map(|row| crate::local_playlist::row_queue_track(&row.item))
                .collect(),
        );
        if tracks.is_empty() {
            return;
        }
        stamp_queue_context(&mut tracks, "playlist", &playlist_id);
        let start_track_id = tracks[0].id;
        runtime.core().set_queue(tracks, Some(0)).await;
        after_track_change(&runtime, &weak, start_track_id).await;
        refresh_sidebar(true);
    });
}

/// Enqueue a whole playlist (by id) at the end of the queue, or immediately
/// after the current track when `next`. Fetches the playlist's tracks fresh,
/// so it works from any playlist CARD (carousels, search, favorites) — not just
/// the currently-open PlaylistView. Mirrors the album enqueue paths.
pub fn enqueue_playlist(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    playlist_id: String,
    next: bool,
) {
    let Ok(pid) = playlist_id.parse::<u64>() else {
        return;
    };
    handle.spawn(async move {
        let playlist = match runtime.core().get_playlist(pid).await {
            Ok(playlist) => playlist,
            Err(e) => {
                log::error!("[qbz-slint] playback: enqueue get_playlist {pid} failed: {e}");
                return;
            }
        };
        let qobuz_tracks: Vec<Track> = playlist.tracks.map(|c| c.items).unwrap_or_default();
        // MIXED playlists (T2 fix-forward, spec §1.3): merge the local
        // sidecar rows at their stored slots so a card/hero enqueue carries
        // EVERY row WITH its source — Tauri's hero arms rebuild catalog-only
        // tracks and drop `source`, crashing auto-advance; our merged
        // rows enqueue as the source-aware QueueTracks the detail plays.
        // Pure-Qobuz playlists read an empty sidecar and are unchanged.
        let qobuz_count = qobuz_tracks.len() as u32;
        let sidecar = tokio::task::spawn_blocking(move || {
            crate::local_playlist::read_sidecar_rows_blocking(pid, qobuz_count)
        })
        .await
        .unwrap_or_default();
        let rows = crate::playlist::interleave_rows(qobuz_tracks, sidecar);
        // Drop blacklisted Qobuz rows (performer; local rows kept by the
        // source guard). Silent early-return when nothing playable remains.
        let tracks: Vec<QueueTrack> = filter_blacklisted_queue(
            rows.iter()
                .filter_map(|row| crate::local_playlist::row_queue_track(&row.item))
                .collect(),
        );
        if tracks.is_empty() {
            return;
        }
        if next {
            // Reverse so the inserted block keeps the playlist's order.
            for track in tracks.into_iter().rev() {
                runtime.core().add_track_next(track).await;
            }
        } else {
            runtime.core().add_tracks(tracks).await;
        }
        refresh_sidebar(false);
        crate::toast::success_weak(
            &weak,
            if next { qbz_i18n::t("Playing next") } else { qbz_i18n::t("Added to queue") },
        );
    });
}
