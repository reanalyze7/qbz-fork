use super::create_test_track;
use super::QueueManager;

// ============ Stop-After Marker — Basic API ============

#[test]
fn test_set_stop_after_stores_marker() {
    let queue = QueueManager::new();
    queue.add_track(create_test_track(101));
    queue.add_track(create_test_track(102));
    queue.add_track(create_test_track(103));

    queue.set_stop_after(102);

    assert_eq!(queue.get_stop_after(), Some(102));
}

#[test]
fn test_set_stop_after_replaces_previous_marker() {
    let queue = QueueManager::new();
    queue.add_track(create_test_track(101));
    queue.add_track(create_test_track(102));

    queue.set_stop_after(101);
    queue.set_stop_after(102);

    assert_eq!(queue.get_stop_after(), Some(102));
}

#[test]
fn test_clear_stop_after_resets_marker() {
    let queue = QueueManager::new();
    queue.add_track(create_test_track(101));
    queue.set_stop_after(101);

    queue.clear_stop_after();

    assert_eq!(queue.get_stop_after(), None);
}

#[test]
fn test_set_stop_after_silently_ignores_unknown_id() {
    let queue = QueueManager::new();
    queue.add_track(create_test_track(101));
    queue.add_track(create_test_track(102));

    queue.set_stop_after(999); // not in queue

    assert_eq!(queue.get_stop_after(), None);
}

#[test]
fn test_set_stop_after_on_empty_queue_is_noop() {
    let queue = QueueManager::new();

    queue.set_stop_after(101);

    assert_eq!(queue.get_stop_after(), None);
}

// ============ Stop-After Marker — Consume (Firing Path) ============

#[test]
fn test_consume_stop_after_if_fires_on_match() {
    let queue = QueueManager::new();
    queue.add_track(create_test_track(101));
    queue.add_track(create_test_track(102));
    queue.set_stop_after(102);

    let fired = queue.consume_stop_after_if(102);

    assert!(fired, "consume should return true on match");
    assert_eq!(queue.get_stop_after(), None, "marker should be cleared after firing");
}

#[test]
fn test_consume_stop_after_if_does_not_fire_on_mismatch() {
    let queue = QueueManager::new();
    queue.add_track(create_test_track(101));
    queue.add_track(create_test_track(102));
    queue.set_stop_after(102);

    let fired = queue.consume_stop_after_if(101);

    assert!(!fired, "consume should return false on mismatch");
    assert_eq!(queue.get_stop_after(), Some(102), "marker should remain on mismatch");
}

#[test]
fn test_consume_stop_after_if_with_no_marker_returns_false() {
    let queue = QueueManager::new();
    queue.add_track(create_test_track(101));

    let fired = queue.consume_stop_after_if(101);

    assert!(!fired);
}
