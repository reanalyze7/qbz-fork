//! Per-row play / play-next / add-to-queue (context-menu actions).

use crate::playback::{after_track_change, refresh_sidebar};
use crate::AppWindow;

use super::load::load_collection;
use super::resolve_item::resolve_single_item;
use super::{Runtime, RowMode};

/// Per-row default **Play** (`on_play_item`) and the context-menu **Play**
/// action: resolve the SINGLE item by `source_item_id`, then replace-play just
/// that item. No queue-source stamp, no touch_play (per-row, not whole
/// collection).
pub fn play_item(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    collection_id: String,
    source_item_id: String,
) {
    item_action(runtime, weak, handle, collection_id, source_item_id, "play".to_string());
}

/// Per-row context-menu action (`on_item_action`): play / play-next /
/// add-to-queue for the single item identified by `source_item_id`.
pub fn item_action(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    collection_id: String,
    source_item_id: String,
    action: String,
) {
    let Some(mode) = RowMode::parse(&action) else {
        log::warn!("[qbz-slint] myqbz_play: unknown item action {action}");
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
            .find(|it| it.source_item_id == source_item_id)
            .cloned()
        else {
            log::warn!(
                "[qbz-slint] myqbz_play: item {source_item_id} not found in collection {collection_id}"
            );
            return;
        };

        let tracks = resolve_single_item(&runtime, &item).await;
        if tracks.is_empty() {
            crate::toast::error_weak(&weak, qbz_i18n::t("This item resolved to 0 playable tracks"));
            return;
        }

        match mode {
            RowMode::Play => {
                // Replace-play this single item — NO queue-source stamp, NO
                // touch_play (per-row).
                let first_id = tracks[0].id;
                runtime.core().set_queue(tracks, Some(0)).await;
                after_track_change(&runtime, &weak, first_id).await;
                refresh_sidebar(true);
            }
            RowMode::PlayNext => {
                // Insert in REVERSE so the first resolved track lands
                // immediately after the current track (spec §9.8).
                for track in tracks.into_iter().rev() {
                    runtime.core().add_track_next(track).await;
                }
                refresh_sidebar(false);
                crate::toast::success_weak(&weak, qbz_i18n::t("Playing next"));
            }
            RowMode::AddToQueue => {
                runtime.core().add_tracks(tracks).await;
                refresh_sidebar(false);
                crate::toast::success_weak(&weak, qbz_i18n::t("Added to queue"));
            }
        }
    });
}
