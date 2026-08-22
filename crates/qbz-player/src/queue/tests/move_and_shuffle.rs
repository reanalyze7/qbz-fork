use super::create_test_track;
use super::QueueManager;

#[test]
fn test_move_track_down_without_current_track() {
    let queue = QueueManager::new();

    for i in 1..=5 {
        queue.add_track(create_test_track(i));
    }

    let result = queue.move_track(0, 3);

    assert!(result, "move_track should succeed");
    assert_eq!(
        queue
            .get_state()
            .upcoming
            .iter()
            .map(|track| track.id)
            .collect::<Vec<u64>>(),
        vec![2, 3, 1, 4, 5]
    );
}

#[test]
fn test_move_track_down_with_current_track() {
    let queue = QueueManager::new();

    for i in 1..=5 {
        queue.add_track(create_test_track(i));
    }
    queue.play_index(0);

    let result = queue.move_track(0, 3);

    assert!(result, "move_track should succeed");
    assert_eq!(
        queue
            .get_state()
            .upcoming
            .iter()
            .map(|track| track.id)
            .collect::<Vec<u64>>(),
        vec![3, 4, 2, 5]
    );
}

#[test]
fn test_move_track_up_without_current_track() {
    let queue = QueueManager::new();

    for i in 1..=5 {
        queue.add_track(create_test_track(i));
    }

    let result = queue.move_track(3, 0);

    assert!(result, "move_track should succeed");
    assert_eq!(
        queue
            .get_state()
            .upcoming
            .iter()
            .map(|track| track.id)
            .collect::<Vec<u64>>(),
        vec![4, 1, 2, 3, 5]
    );
}

#[test]
fn test_move_track_up_with_current_track() {
    let queue = QueueManager::new();

    for i in 1..=5 {
        queue.add_track(create_test_track(i));
    }
    queue.play_index(0);

    let result = queue.move_track(3, 0);

    assert!(result, "move_track should succeed");
    assert_eq!(
        queue
            .get_state()
            .upcoming
            .iter()
            .map(|track| track.id)
            .collect::<Vec<u64>>(),
        vec![5, 2, 3, 4]
    );
}
