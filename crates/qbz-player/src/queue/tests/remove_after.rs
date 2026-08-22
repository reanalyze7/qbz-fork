use super::create_test_track;
use super::QueueManager;

#[test]
fn test_remove_after_returns_count() {
    let queue = QueueManager::new();
    for id in [101, 102, 103, 104, 105] {
        queue.add_track(create_test_track(id));
    }

    let removed = queue.remove_after(1);

    assert_eq!(removed, 3, "should remove indices 2, 3, 4");
    let state = queue.get_state();
    assert_eq!(state.total_tracks, 2);
}

#[test]
fn test_remove_after_on_last_index_is_noop() {
    let queue = QueueManager::new();
    for id in [101, 102, 103] {
        queue.add_track(create_test_track(id));
    }

    let removed = queue.remove_after(2);

    assert_eq!(removed, 0);
    assert_eq!(queue.get_state().total_tracks, 3);
}

#[test]
fn test_remove_after_with_index_out_of_bounds_is_noop() {
    let queue = QueueManager::new();
    queue.add_track(create_test_track(101));
    queue.add_track(create_test_track(102));

    let removed = queue.remove_after(99);

    assert_eq!(removed, 0);
    assert_eq!(queue.get_state().total_tracks, 2);
}

#[test]
fn test_remove_after_invalidates_marker_when_in_removed_range() {
    let queue = QueueManager::new();
    for id in [101, 102, 103, 104] {
        queue.add_track(create_test_track(id));
    }
    queue.set_stop_after(103);

    queue.remove_after(1); // removes 103, 104

    assert_eq!(queue.get_stop_after(), None);
}

#[test]
fn test_remove_after_keeps_marker_when_before_range() {
    let queue = QueueManager::new();
    for id in [101, 102, 103, 104] {
        queue.add_track(create_test_track(id));
    }
    queue.set_stop_after(101);

    queue.remove_after(2); // removes 104 only (index 3)

    assert_eq!(queue.get_stop_after(), Some(101));
}

#[test]
fn test_remove_after_keeps_marker_when_at_pivot_index() {
    let queue = QueueManager::new();
    for id in [101, 102, 103, 104] {
        queue.add_track(create_test_track(id));
    }
    queue.set_stop_after(102);

    queue.remove_after(1); // removes indices 2, 3 — track 102 (at index 1) stays

    assert_eq!(queue.get_stop_after(), Some(102));
}
