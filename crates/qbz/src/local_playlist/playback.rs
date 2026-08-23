//! Playback: play-all/play-from-row/enqueue for the open local playlist.

use std::collections::HashMap;

use qbz_models::QueueTrack;
use slint::{ComponentHandle, Model};

use super::state::{CURRENT_META, CURRENT_QUEUE};
use super::Runtime;
use crate::playback::{after_track_change, refresh_sidebar};
use crate::{AppWindow, PlaylistState};

/// Replace the queue with `tracks`, stamp the offline-only flag (D8 guard:
/// the QConnect push site reads it and skips the cloud), start at `start`.
async fn play_stamped(runtime: &Runtime, weak: &slint::Weak<AppWindow>, tracks: Vec<QueueTrack>, start: usize) {
    if tracks.is_empty() {
        crate::toast::error_weak(weak, qbz_i18n::t("Nothing playable in this playlist right now"));
        return;
    }
    let offline_only = CURRENT_META
        .lock()
        .ok()
        .and_then(|m| m.as_ref().map(|(_, o)| *o))
        .unwrap_or(false);
    let start = start.min(tracks.len() - 1);
    let first_id = tracks[start].id;
    runtime.core().set_queue(tracks, Some(start)).await;
    // AFTER set_queue (which clears the stamp on every replacement).
    runtime.core().set_queue_offline_only(offline_only);
    after_track_change(runtime, weak, first_id).await;
    refresh_sidebar(true);
}

/// Order the queue snapshot by the VISIBLE row order (sort/search applied),
/// mirroring `playback`'s visible-order rule for the Qobuz detail.
fn visible_ordered_queue(window: &AppWindow) -> Vec<QueueTrack> {
    let snapshot = CURRENT_QUEUE.lock().map(|q| q.clone()).unwrap_or_default();
    let by_id: HashMap<String, &QueueTrack> =
        snapshot.iter().map(|q| (q.id.to_string(), q)).collect();
    let model = window.global::<PlaylistState>().get_tracks();
    let mut out: Vec<QueueTrack> = Vec::new();
    for i in 0..model.row_count() {
        if let Some(it) = model.row_data(i) {
            if let Some(q) = by_id.get(it.id.as_str()) {
                out.push((*q).clone());
            }
        }
    }
    if out.is_empty() {
        snapshot
    } else {
        out
    }
}

/// Hero Play (visible order) / Shuffle for the open local playlist.
pub fn play_all(
    window: &AppWindow,
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    shuffle: bool,
) {
    let mut tracks = visible_ordered_queue(window);
    if shuffle {
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
    }
    let ctx_id = window.global::<PlaylistState>().get_id().to_string();
    crate::playback::stamp_queue_context(&mut tracks, "playlist", &ctx_id);
    handle.spawn(async move {
        play_stamped(&runtime, &weak, tracks, 0).await;
    });
}

/// Per-row "play from here" — queue the visible order starting at the
/// clicked row (the local branch of `play_track_in_context`'s Playlist arm).
/// Returns false when the clicked row is not in the playable snapshot.
pub fn play_from_visible(
    window: &AppWindow,
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    clicked_id: &str,
) -> bool {
    let mut tracks = visible_ordered_queue(window);
    let Some(idx) = tracks.iter().position(|q| q.id.to_string() == clicked_id) else {
        return false;
    };
    let ctx_id = window.global::<PlaylistState>().get_id().to_string();
    crate::playback::stamp_queue_context(&mut tracks, "playlist", &ctx_id);
    handle.spawn(async move {
        play_stamped(&runtime, &weak, tracks, idx).await;
    });
    true
}
