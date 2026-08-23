//! Batch enqueue of already-fetched catalog tracks, plus resolving a list
//! of ids and enqueueing them.

use super::super::queue_context::make_queue_track;
use super::super::recent_blacklist::track_is_blacklisted_full;
use super::super::state::refresh_sidebar;
use super::super::Runtime;
use crate::AppWindow;

/// Resolve a list of Qobuz track ids (order-preserving) and enqueue them at the
/// end of the queue (or play-next). Backs the external-reco Weekly rows'
/// "add to queue" button (P7). Toasts the outcome.
pub fn enqueue_track_ids(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    ids: Vec<u64>,
    next: bool,
) {
    if ids.is_empty() {
        return;
    }
    handle.spawn(async move {
        match runtime.core().get_tracks_batch(&ids).await {
            Ok(tracks) if !tracks.is_empty() => {
                let n = tracks.len();
                enqueue_tracks(runtime, tokio::runtime::Handle::current(), tracks, next);
                crate::toast::success_weak(
                    &weak,
                    qbz_i18n::tf(
                        "Added {} track to queue",
                        "Added {} tracks to queue",
                        n as i64,
                        &[&n.to_string()],
                    ),
                );
            }
            Ok(_) => crate::toast::error_weak(&weak, qbz_i18n::t("No tracks to add")),
            Err(e) => {
                log::error!("[qbz-slint] enqueue_track_ids: get_tracks_batch failed: {e}");
                crate::toast::error_weak(&weak, qbz_i18n::t("Failed to add tracks"));
            }
        }
    });
}

/// Append (or insert-next) a batch of already-fetched tracks to the queue
/// without re-fetching them. Used by the favorites bulk bar.
pub fn enqueue_tracks(
    runtime: Runtime,
    handle: tokio::runtime::Handle,
    tracks: Vec<qbz_models::Track>,
    next: bool,
) {
    if tracks.is_empty() {
        return;
    }
    // Drop blacklisted tracks (performer OR composer — D-FEAT) from the bulk
    // batch before routing/enqueueing. Silent early-return when 0 remain.
    let tracks: Vec<qbz_models::Track> = tracks
        .into_iter()
        .filter(|track| !track_is_blacklisted_full(track, None))
        .collect();
    if tracks.is_empty() {
        return;
    }
    handle.spawn(async move {
        // For "play next" each insert lands right after the current track,
        // so reverse the batch to preserve the selection's order.
        let ordered: Vec<qbz_models::Track> = if next {
            tracks.into_iter().rev().collect()
        } else {
            tracks
        };
        for track in ordered {
            let (album_id, album_title, album_artwork) = track
                .album
                .as_ref()
                .map(|a| (a.id.clone(), a.title.clone(), a.image.best().cloned().unwrap_or_default()))
                .unwrap_or_default();
            let album_artist = track.performer.as_ref().map(|p| p.name.clone()).unwrap_or_default();
            let qt =
                make_queue_track(&track, &album_id, &album_title, &album_artist, &album_artwork, None);
            if next {
                runtime.core().add_track_next(qt).await;
            } else {
                runtime.core().add_track(qt).await;
            }
        }
        refresh_sidebar(false);
    });
}
