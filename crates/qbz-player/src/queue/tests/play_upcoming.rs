use super::create_test_track;
use super::QueueManager;

#[test]
fn test_play_upcoming_at_without_shuffle_uses_linear_offset() {
    let queue = QueueManager::new();
    for i in 1..=5 {
        queue.add_track(create_test_track(i));
    }
    queue.play_index(1); // current = track id 2

    // upcoming list is [3, 4, 5]; clicking position 1 must play id 4
    let track = queue.play_upcoming_at(1).expect("track");
    assert_eq!(track.id, 4);
}

#[test]
fn test_play_upcoming_at_with_shuffle_follows_shuffle_order() {
    let queue = QueueManager::new();
    for i in 1..=5 {
        queue.add_track(create_test_track(i));
    }

    // Authoritative shuffle: playing head is shuffle[0]=2 (id 3),
    // upcoming order becomes [5, 2, 4, 1] (track ids).
    queue.set_queue_with_order(
        (1..=5).map(create_test_track).collect(),
        Some(2),
        true,
        Some(vec![2, 4, 1, 3, 0]),
    );

    let state = queue.get_state();
    assert_eq!(
        state.upcoming.iter().map(|t| t.id).collect::<Vec<_>>(),
        vec![5, 2, 4, 1]
    );

    // Clicking upcoming position 2 must play track id 4, not id 5
    // (which would be the "current_index + 2 + 1" = 5 broken path).
    let track = queue.play_upcoming_at(2).expect("track");
    assert_eq!(track.id, 4);
}

#[test]
fn test_set_shuffle_with_order_uses_authoritative_order() {
    let queue = QueueManager::new();
    for i in 1..=5 {
        queue.add_track(create_test_track(i));
    }

    queue.play_index(2);
    queue.set_shuffle_with_order(true, Some(vec![2, 4, 1, 3, 0]));

    let state = queue.get_state();
    assert!(state.shuffle);
    assert_eq!(
        state.upcoming.iter().map(|track| track.id).collect::<Vec<_>>(),
        vec![5, 2, 4, 1]
    );
}

#[test]
fn test_set_shuffle_with_order_preserves_current_order_when_invalid() {
    let queue = QueueManager::new();
    for i in 1..=4 {
        queue.add_track(create_test_track(i));
    }

    queue.play_index(1);
    queue.set_shuffle_with_order(true, Some(vec![1, 1, 2, 3]));

    let state = queue.get_state();
    assert!(state.shuffle);
    assert_eq!(
        state.upcoming.iter().map(|track| track.id).collect::<Vec<_>>(),
        vec![3, 4]
    );
}

#[test]
fn test_set_queue_with_order_applies_authoritative_shuffle_before_snapshot() {
    let queue = QueueManager::new();
    let tracks = (1..=5).map(create_test_track).collect::<Vec<_>>();

    queue.set_queue_with_order(tracks, Some(0), true, Some(vec![0, 3, 1, 4, 2]));

    let state = queue.get_state();
    assert!(state.shuffle);
    assert_eq!(
        state.upcoming.iter().map(|track| track.id).collect::<Vec<_>>(),
        vec![4, 2, 5, 3]
    );
}

#[test]
fn test_set_queue_with_order_preserves_queue_order_when_authoritative_order_missing() {
    let queue = QueueManager::new();
    let tracks = (1..=5).map(create_test_track).collect::<Vec<_>>();

    queue.set_queue_with_order(tracks, Some(1), true, None);

    let state = queue.get_state();
    assert!(state.shuffle);
    assert_eq!(
        state.upcoming.iter().map(|track| track.id).collect::<Vec<_>>(),
        vec![3, 4, 5]
    );
}
