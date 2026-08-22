use super::QueueManager;
use qbz_models::QueueTrack;

mod clear;
mod history_sync;
mod move_and_shuffle;
mod play_upcoming;
mod queue_history_316_a;
mod queue_history_316_b;
mod remove_after;
mod remove_upcoming_after;
mod shuffle_reorder;
mod state_view_tests;
mod stop_after_basic;
mod stop_after_invalidation;

/// Helper: build a queue with N tracks, play track 0, advance through
/// `advance_count` to populate history, returning the queue.
pub(super) fn queue_with_played_history(track_count: u64, advance_count: usize) -> QueueManager {
    let queue = QueueManager::new();
    for i in 1..=track_count {
        queue.add_track(create_test_track(i));
    }
    queue.play_index(0);
    for _ in 0..advance_count {
        queue.next();
    }
    queue
}

pub(super) fn create_test_track(id: u64) -> QueueTrack {
    QueueTrack {
        id,
        title: format!("Track {}", id),
        version: None,
        artist: "Artist".to_string(),
        album: "Album".to_string(),
        album_version: None,
        duration_secs: 180,
        artwork_url: None,
        hires: false,
        bit_depth: None,
        sample_rate: None,
        is_local: false,
        album_id: None,
        artist_id: None,
        streamable: true,
        source: Some("test".to_string()),
        parental_warning: false,
        source_item_id_hint: None,
        context_kind: None,
        context_id: None,
    }
}
