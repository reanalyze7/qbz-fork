//! Expanded-view single-track actions (play now / play next / play later).

use crate::playback::{after_track_change, refresh_sidebar};
use crate::AppWindow;

use super::load::load_collection;
use super::resolve_item::fetch_item_tracks;
use super::Runtime;

/// The inline-track menu mode (expanded view-mode TrackRow actions, spec §8
/// `menuActions`): play-now / play-next / play-later for ONE track resolved
/// from its parent item. (go-to-album routes through `open-item` in main.rs,
/// not here.)
enum InlineTrackMode {
    Play,
    PlayNext,
    PlayLater,
}

impl InlineTrackMode {
    fn parse(action: &str) -> Option<Self> {
        match action {
            "play" => Some(Self::Play),
            "play-next" | "play_next" => Some(Self::PlayNext),
            "play-later" | "play_later" | "queue" | "add-to-queue" | "append" => {
                Some(Self::PlayLater)
            }
            _ => None,
        }
    }
}

/// Play / queue a SINGLE inline track from an expanded item (spec 12 §8
/// `onPlayTrackFromItem` / `onPlayTrackNext` / `onPlayTrackLater`). Re-resolves
/// the parent item's tracks (the inline view holds only display rows, not the
/// numeric `QueueTrack`s) and selects the one matching `track_id`:
/// - **Play**: replace-play just that track (no queue-source stamp, per-row).
/// - **PlayNext**: insert immediately after the current track.
/// - **PlayLater**: append at the end of the queue.
pub fn play_inline_track(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    collection_id: String,
    item_source_item_id: String,
    track_id: String,
    action: String,
) {
    let Some(mode) = InlineTrackMode::parse(&action) else {
        log::warn!("[qbz-slint] myqbz_play: unknown inline-track action {action}");
        return;
    };
    let Ok(track_id) = track_id.parse::<u64>() else {
        log::warn!("[qbz-slint] myqbz_play: inline-track non-numeric id {track_id}");
        return;
    };
    handle.spawn(async move {
        let Some(collection) = load_collection(&collection_id).await else {
            crate::toast::error_weak(&weak, qbz_i18n::t("Couldn't load this collection"));
            return;
        };
        let Some(item) = collection
            .items
            .iter()
            .find(|it| it.source_item_id == item_source_item_id)
            .cloned()
        else {
            log::warn!(
                "[qbz-slint] myqbz_play: inline-track item {item_source_item_id} not found"
            );
            return;
        };

        let tracks = fetch_item_tracks(&runtime, &item).await;
        let Some(track) = tracks.into_iter().find(|t| t.id == track_id) else {
            crate::toast::error_weak(&weak, qbz_i18n::t("This track is no longer available"));
            return;
        };

        match mode {
            InlineTrackMode::Play => {
                let first_id = track.id;
                runtime.core().set_queue(vec![track], Some(0)).await;
                after_track_change(&runtime, &weak, first_id).await;
                refresh_sidebar(true);
            }
            InlineTrackMode::PlayNext => {
                runtime.core().add_track_next(track).await;
                refresh_sidebar(false);
                crate::toast::success_weak(&weak, qbz_i18n::t("Playing next"));
            }
            InlineTrackMode::PlayLater => {
                runtime.core().add_tracks(vec![track]).await;
                refresh_sidebar(false);
                crate::toast::success_weak(&weak, qbz_i18n::t("Added to queue"));
            }
        }
    });
}
