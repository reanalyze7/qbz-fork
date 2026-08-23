//! Selective enqueue and click-to-play-from for the artist Popular list.

use super::artist_fetch::fetch_artist_top_for_play;
use super::super::engine::after_track_change;
use super::super::queue_context::stamp_queue_context;
use super::super::state::refresh_sidebar;
use super::super::Runtime;
use crate::AppWindow;
use qbz_models::QueueTrack;

/// Enqueue (play-next or append) a subset of the artist's Popular tracks,
/// identified by catalog id. Re-fetches the page (like the play-all path),
/// filters to `ids`, preserves the page order, and queues — QConnect-aware
/// (mirrors `enqueue_queue_tracks`). Drives both the bulk bar (selection)
/// and the section "more" menu (all ids).
pub fn enqueue_artist_top_selected(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    artist_id: String,
    ids: Vec<String>,
    next: bool,
) {
    if ids.is_empty() {
        return;
    }
    handle.spawn(async move {
        let Some(all) = fetch_artist_top_for_play(&runtime, &weak, &artist_id).await else {
            return;
        };
        let want: std::collections::HashSet<u64> =
            ids.iter().filter_map(|s| s.parse::<u64>().ok()).collect();
        let tracks: Vec<QueueTrack> = all.into_iter().filter(|qt| want.contains(&qt.id)).collect();
        if tracks.is_empty() {
            return;
        }
        if next {
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

/// Play the artist's Popular tracks starting at the clicked track id (queues
/// the tracks that follow it). `visible_ids` is the Popular-tracks VISIBLE row
/// order — the queue is reordered/filtered to match, so the in-page search
/// filter is respected. Re-fetches the page like `play_artist_top_tracks`.
pub fn play_artist_top_from(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    artist_id: String,
    visible_ids: Vec<String>,
    clicked_id: String,
) {
    handle.spawn(async move {
        let Some(tracks) = fetch_artist_top_for_play(&runtime, &weak, &artist_id).await else {
            return;
        };
        let mut tracks = super::super::queue_build::reorder_queue_by_visible(tracks, &visible_ids);
        stamp_queue_context(&mut tracks, "artist", &artist_id);
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
