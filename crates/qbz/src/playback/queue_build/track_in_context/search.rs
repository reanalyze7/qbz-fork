//! The `ContentView::Search` branch of `play_track_in_context`.
use slint::ComponentHandle;

use crate::playback::queue_build::model_helpers::track_item_to_queue;
use crate::playback::queue_build::from_model::queue_from_model;
use crate::playback::queue_build::play_queue::play_queue;
use crate::playback::recent_blacklist::filter_blacklisted_queue;
use crate::playback::Runtime;
use crate::{AppWindow, SearchState};

/// Search keeps no full-Track cache — build the queue straight off
/// the visible model (Qobuz tracks resolve by id at play time).
pub(super) fn handle(
    window: &AppWindow,
    runtime: &Runtime,
    weak: &slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    clicked_id: &str,
) -> bool {
    let model = window.global::<SearchState>().get_tracks();
    let (queue, found) = queue_from_model(&model, clicked_id);
    if found.is_some() {
        // Drop blacklisted rows that visually follow the clicked track,
        // then re-anchor the start on the clicked id (it can't itself be
        // blacklisted — greyed rows are inert). Empty => nothing to do.
        let queue = filter_blacklisted_queue(queue);
        if let Some(idx) = queue.iter().position(|q| q.id.to_string() == clicked_id) {
            play_queue(runtime.clone(), weak.clone(), handle.clone(), queue, idx);
        }
        return true;
    }
    // The "most popular" hero is a top-track card, not a results row.
    // Play it as the queue head, then the visible results, so it acts
    // like a first-class track (clicking it queues what follows).
    let ss = window.global::<SearchState>();
    if ss.get_most_popular_kind().as_str() == "track" {
        let hero = ss.get_most_popular_track();
        if hero.id.as_str() == clicked_id {
            if let Some(hq) = track_item_to_queue(&hero) {
                // Filter the trailing results; keep the hero at the head.
                let mut q = filter_blacklisted_queue(queue);
                q.insert(0, hq);
                play_queue(runtime.clone(), weak.clone(), handle.clone(), q, 0);
                return true;
            }
        }
    }
    false
}
