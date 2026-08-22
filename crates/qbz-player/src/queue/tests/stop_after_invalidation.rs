use super::create_test_track;
use super::QueueManager;

// ============ Stop-After Marker — Invalidation on Queue Mutations ============

#[test]
fn test_set_queue_invalidates_marker() {
    let queue = QueueManager::new();
    queue.add_track(create_test_track(101));
    queue.add_track(create_test_track(102));
    queue.set_stop_after(102);

    queue.set_queue(vec![create_test_track(201), create_test_track(202)], None);

    assert_eq!(queue.get_stop_after(), None);
}

#[test]
fn test_clear_invalidates_marker() {
    let queue = QueueManager::new();
    queue.add_track(create_test_track(101));
    queue.add_track(create_test_track(102));
    queue.set_stop_after(102);

    queue.clear(true);

    assert_eq!(queue.get_stop_after(), None);
}

#[test]
fn test_remove_track_invalidates_marker_when_marked_track_removed() {
    let queue = QueueManager::new();
    queue.add_track(create_test_track(101));
    queue.add_track(create_test_track(102));
    queue.add_track(create_test_track(103));
    queue.set_stop_after(102);

    queue.remove_track(1); // removes track 102

    assert_eq!(queue.get_stop_after(), None);
}

#[test]
fn test_remove_track_keeps_marker_when_other_track_removed() {
    let queue = QueueManager::new();
    queue.add_track(create_test_track(101));
    queue.add_track(create_test_track(102));
    queue.add_track(create_test_track(103));
    queue.set_stop_after(102);

    queue.remove_track(0); // removes track 101

    assert_eq!(queue.get_stop_after(), Some(102));
}

#[test]
fn test_move_track_does_not_invalidate_marker() {
    let queue = QueueManager::new();
    queue.add_track(create_test_track(101));
    queue.add_track(create_test_track(102));
    queue.add_track(create_test_track(103));
    queue.set_stop_after(102);

    queue.move_track(1, 0); // 102 moves to position 0

    assert_eq!(queue.get_stop_after(), Some(102));
}
