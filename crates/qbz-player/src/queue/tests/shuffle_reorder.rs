use super::create_test_track;
use super::QueueManager;

#[test]
fn test_move_track_with_shuffle_reorders_shuffle_timeline() {
    let queue = QueueManager::new();
    for i in 1..=8 {
        queue.add_track(create_test_track(i));
    }

    queue.play_index(0);
    queue.set_shuffle(true);

    let before_shuffle = {
        let state = queue.state.lock().unwrap();
        state.shuffle_order.clone()
    };

    // With current_index=0 and shuffle_position=0:
    // upcoming move 2 -> 0 maps to shuffle positions 3 -> 1.
    assert!(queue.move_track(2, 0));

    let after_shuffle = {
        let state = queue.state.lock().unwrap();
        state.shuffle_order.clone()
    };

    let mut expected = before_shuffle.clone();
    let moved = expected.remove(3);
    expected.insert(1, moved);

    assert_eq!(after_shuffle, expected);
    assert_eq!(after_shuffle.len(), 8);
}

#[test]
fn test_remove_track_with_shuffle_preserves_shuffle_order() {
    let queue = QueueManager::new();
    for i in 1..=8 {
        queue.add_track(create_test_track(i));
    }

    queue.play_index(0);
    queue.set_shuffle(true);

    let before_shuffle = {
        let state = queue.state.lock().unwrap();
        state.shuffle_order.clone()
    };

    assert!(queue.remove_track(2).is_some());

    let after_shuffle = {
        let state = queue.state.lock().unwrap();
        state.shuffle_order.clone()
    };

    let expected: Vec<usize> = before_shuffle
        .into_iter()
        .filter(|&idx| idx != 2)
        .map(|idx| if idx > 2 { idx - 1 } else { idx })
        .collect();

    assert_eq!(after_shuffle, expected);
    assert_eq!(after_shuffle.len(), 7);
}

#[test]
fn test_enabling_shuffle_keeps_all_remaining_tracks_upcoming() {
    let queue = QueueManager::new();
    for i in 1..=11 {
        queue.add_track(create_test_track(i));
    }

    queue.play_index(0);
    queue.set_shuffle(true);

    let state = queue.get_state();
    assert_eq!(state.total_tracks, 11);
    assert_eq!(state.upcoming.len(), 10);
}
