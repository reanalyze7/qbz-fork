use super::create_test_track;
use super::QueueManager;

#[test]
fn test_get_state_includes_stop_after() {
    let queue = QueueManager::new();
    queue.add_track(create_test_track(101));
    queue.set_stop_after(101);

    let state = queue.get_state();

    assert_eq!(state.stop_after_track_id, Some(101));
}

#[test]
fn test_get_state_full_returns_uncapped_upcoming() {
    let queue = QueueManager::new();
    // 50 tracks — more than get_state()'s 20-track upcoming cap.
    for i in 1..=50 {
        queue.add_track(create_test_track(i));
    }
    queue.play_index(0);

    let capped = queue.get_state();
    assert_eq!(capped.upcoming.len(), 20, "get_state caps upcoming at 20");

    let full = queue.get_state_full();
    assert_eq!(full.upcoming.len(), 49, "get_state_full returns all upcoming");
    assert_eq!(full.total_tracks, 50);
    assert_eq!(full.upcoming.first().unwrap().id, 2);
    assert_eq!(full.upcoming.last().unwrap().id, 50);
}

#[test]
fn test_get_state_full_returns_uncapped_history() {
    let queue = QueueManager::new();
    for i in 1..=30 {
        queue.add_track(create_test_track(i));
    }
    queue.play_index(0);
    // Advance through 25 tracks — more than get_state()'s 10-entry cap.
    for _ in 0..25 {
        queue.next();
    }

    let capped = queue.get_state();
    assert_eq!(capped.history.len(), 10, "get_state caps history at 10");

    let full = queue.get_state_full();
    assert_eq!(full.history.len(), 25, "get_state_full returns all history");
    // Newest-first ordering: most recently played sits at the front.
    assert_eq!(full.history.first().unwrap().id, 25);
}

#[test]
fn test_get_state_full_no_current_track_returns_all_as_upcoming() {
    let queue = QueueManager::new();
    for i in 1..=5 {
        queue.add_track(create_test_track(i));
    }

    let full = queue.get_state_full();
    assert!(full.current_track.is_none());
    assert_eq!(full.upcoming.len(), 5);
}

#[test]
fn test_get_state_returns_none_when_no_marker() {
    let queue = QueueManager::new();
    queue.add_track(create_test_track(101));

    let state = queue.get_state();

    assert_eq!(state.stop_after_track_id, None);
}
