//! `play`: replace the queue with the rail, starting at the clicked row.

use qbz_models::QueueTrack;

use crate::adapter::SlintAdapter;
use crate::AppWindow;

use super::state::RAIL_QUEUE;

type Runtime = std::sync::Arc<qbz_app::shell::AppRuntime<SlintAdapter>>;

/// Play the rail starting at the clicked track id: the rail becomes the
/// queue (replace), playback starts at the clicked row and continues down
/// the list through the existing offline-capable play path.
pub fn play(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    id: String,
) {
    let queue: Vec<QueueTrack> = RAIL_QUEUE
        .lock()
        .map(|q| q.clone())
        .unwrap_or_default();
    if queue.is_empty() {
        return;
    }
    let start = id
        .parse::<u64>()
        .ok()
        .and_then(|tid| queue.iter().position(|t| t.id == tid))
        .unwrap_or(0);
    let first_id = queue[start].id;
    handle.spawn(async move {
        runtime.core().set_queue(queue, Some(start)).await;
        crate::playback::after_track_change(&runtime, &weak, first_id).await;
        crate::playback::refresh_sidebar(true);
    });
}
