use super::create_test_track;
use super::QueueManager;

#[test]
fn test_clear_without_current_track() {
    let queue = QueueManager::new();

    queue.add_track(create_test_track(123));
    queue.add_track(create_test_track(124));
    queue.add_track(create_test_track(125));

    queue.clear(true);

    let state = queue.get_state();
    assert!(state.current_track.is_none());
    assert!(state.upcoming.is_empty());
    assert_eq!(state.total_tracks, 0);
}

#[test]
fn test_clear_keeps_current_track() {
    let queue = QueueManager::new();

    queue.add_track(create_test_track(123));
    queue.add_track(create_test_track(124));
    queue.add_track(create_test_track(125));
    queue.play_index(0);

    queue.clear(true);

    let state = queue.get_state();
    assert!(state.current_track.is_some());
    assert_eq!(state.current_track.unwrap().id, 123);
    assert!(state.upcoming.is_empty());
    assert_eq!(state.total_tracks, 1);
}

/// Regression: clear(true) must keep the track at `current_index`, not
/// always `tracks[0]`. Mid-album "Clear queue" previously left the first
/// row as now-playing while audio kept the real current track.
#[test]
fn test_clear_keeps_mid_queue_current_track() {
    let queue = QueueManager::new();

    queue.add_track(create_test_track(100));
    queue.add_track(create_test_track(200));
    queue.add_track(create_test_track(300));
    queue.play_index(1); // current = 200

    queue.clear(true);

    let state = queue.get_state();
    assert!(state.current_track.is_some());
    assert_eq!(state.current_track.unwrap().id, 200);
    assert!(state.upcoming.is_empty());
    assert_eq!(state.total_tracks, 1);
}

#[test]
fn test_clear_wipes_current_track_when_not_kept() {
    let queue = QueueManager::new();

    queue.add_track(create_test_track(123));
    queue.add_track(create_test_track(124));
    queue.play_index(0);

    // keep_current: false — user pressed Clear Queue while nothing was
    // actively playing, so the stale "now playing" slot should go too.
    queue.clear(false);

    let state = queue.get_state();
    assert!(state.current_track.is_none());
    assert!(state.upcoming.is_empty());
    assert_eq!(state.total_tracks, 0);
}

#[test]
fn test_clear_preserves_history() {
    let queue = QueueManager::new();

    queue.add_track(create_test_track(123));
    queue.add_track(create_test_track(124));
    queue.add_track(create_test_track(125));
    queue.play_index(0);
    queue.next(); // push 123 into history, current becomes 124

    let before = queue.get_state();
    assert_eq!(before.history.len(), 1);
    assert_eq!(before.history[0].id, 123);
    assert_eq!(before.current_track.as_ref().map(|t| t.id), Some(124));

    queue.clear(true);

    let after = queue.get_state();
    // Kept track is the one that was current (124), not tracks[0] (123).
    assert_eq!(after.current_track.as_ref().map(|t| t.id), Some(124));
    assert_eq!(after.total_tracks, 1);
    // History is index-based: entries for tracks that left the queue are
    // dropped (same remap-by-id path as set_queue). Under the old
    // truncate(1) bug, 123 stayed as the sole row so history still
    // "resolved" — that was an accident of the bug, not the contract.
    assert!(
        after.history.is_empty(),
        "history must not resolve removed tracks after clear: {:?}",
        after.history.iter().map(|t| t.id).collect::<Vec<_>>()
    );
}
