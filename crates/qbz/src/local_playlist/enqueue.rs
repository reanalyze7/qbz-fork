//! Queue / play-next a local playlist by id (sidebar + now-playing context
//! actions).

use qbz_models::QueueTrack;

use super::Runtime;
use crate::playback::refresh_sidebar;
use crate::AppWindow;

/// When the playlist is OFFLINE-ONLY the queue gets stamped even on append
/// (D8 strict reading: not even its numeric track ids may reach the
/// QConnect cloud push) — the stamp clears on the next replacement.
pub fn enqueue_by_id(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    playlist_id: String,
    play_next: bool,
) {
    handle.spawn(async move {
        let Some(data) = super::detail_local::load(&runtime, &playlist_id).await else {
            crate::toast::error_weak(&weak, qbz_i18n::t("Couldn't load this playlist"));
            return;
        };
        let tracks: Vec<QueueTrack> = data
            .rows
            .iter()
            .filter_map(|r| super::row::row_queue_track(&r.item))
            .collect();
        if tracks.is_empty() {
            crate::toast::error_weak(&weak, qbz_i18n::t("Nothing playable in this playlist right now"));
            return;
        }
        if play_next {
            for track in tracks.into_iter().rev() {
                runtime.core().add_track_next(track).await;
            }
        } else {
            runtime.core().add_tracks(tracks).await;
        }
        if data.offline_only {
            runtime.core().set_queue_offline_only(true);
        }
        refresh_sidebar(false);
        crate::toast::success_weak(
            &weak,
            if play_next {
                qbz_i18n::t("Playing next")
            } else {
                qbz_i18n::t("Added to queue")
            },
        );
    });
}
