//! Hero Play + the shared replace+stamp+touch_play tail (shuffle lives in
//! `hero_shuffle.rs`).

use qbz_models::QueueTrack;

use crate::playback::{after_track_change, refresh_sidebar};
use crate::AppWindow;

use super::load::load_collection;
use super::resolve::resolve_collection;
use super::Runtime;

/// Best-effort `repo::touch_play` (bumps last_played_at + play_count). Errors
/// ignored, exactly like the Tauri command. Runs synchronously via `with_db` —
/// safe to call from the async context (no `&Connection` crosses an `.await`).
pub(super) fn touch_play(collection_id: &str) {
    let _ = crate::library_db::with_db(|db| {
        Ok(db.with_connection(|conn| {
            if let Err(e) = qbz_mixtape::repo::touch_play(conn, collection_id) {
                log::debug!("[qbz-slint] myqbz_play: touch_play({collection_id}) failed: {e}");
            }
        }))
    });
}

/// Replace the queue with `tracks`, start at index 0, stamp the queue-source
/// collection, and `touch_play`. Shared by hero Play + hero Shuffle (the two
/// whole-collection replace paths). Empty `tracks` → toast + no-op.
pub(crate) async fn play_all_tracks(
    runtime: &Runtime,
    weak: &slint::Weak<AppWindow>,
    collection_id: &str,
    tracks: Vec<QueueTrack>,
) {
    if tracks.is_empty() {
        crate::toast::error_weak(weak, qbz_i18n::t("This collection resolved to 0 playable tracks"));
        return;
    }
    let first_id = tracks[0].id;
    runtime.core().set_queue(tracks, Some(0)).await;
    // Queue-source stamp: ONLY on replace (spec §9.9) — this IS a replace.
    runtime
        .runtime()
        .set_queue_source_collection(Some(collection_id.to_string()))
        .await;
    after_track_change(runtime, weak, first_id).await;
    // touch_play is best-effort, replace-only.
    touch_play(collection_id);
    refresh_sidebar(true);
}

/// Hero **Play** (`on_play_all`): resolve the whole collection with its
/// persisted `play_mode`, then replace-play.
pub fn play_all(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    collection_id: String,
) {
    handle.spawn(async move {
        let Some(collection) = load_collection(&collection_id).await else {
            crate::toast::error_weak(&weak, qbz_i18n::t("Couldn't load this collection"));
            return;
        };
        let tracks = resolve_collection(&runtime, &collection, false).await;
        play_all_tracks(&runtime, &weak, &collection_id, tracks).await;
    });
}
