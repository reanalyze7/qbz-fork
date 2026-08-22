use super::create_test_track;
use super::QueueManager;

#[test]
fn test_remove_upcoming_after_linear() {
    let queue = QueueManager::new();
    // 101 playing, upcoming = [102, 103, 104, 105].
    queue.set_queue((101..=105).map(create_test_track).collect(), Some(0));

    // Keep upcoming positions 0..=1 (102, 103); drop 2, 3 (104, 105).
    let removed = queue.remove_upcoming_after(1);

    assert_eq!(removed, 2);
    let state = queue.get_state_full();
    assert_eq!(state.current_track.map(|t| t.id), Some(101));
    assert_eq!(
        state.upcoming.iter().map(|t| t.id).collect::<Vec<_>>(),
        vec![102, 103]
    );
}

#[test]
fn test_remove_upcoming_after_on_last_upcoming_is_noop() {
    let queue = QueueManager::new();
    queue.set_queue((101..=103).map(create_test_track).collect(), Some(0));
    // Upcoming = [102, 103]; position 1 is the last, nothing after it.
    let removed = queue.remove_upcoming_after(1);
    assert_eq!(removed, 0);
    assert_eq!(queue.get_state().total_tracks, 3);
}

#[test]
fn test_remove_upcoming_after_no_current_track() {
    let queue = QueueManager::new();
    // No current index -> every track is upcoming.
    queue.set_queue((101..=104).map(create_test_track).collect(), None);
    let removed = queue.remove_upcoming_after(0);
    assert_eq!(removed, 3);
    let state = queue.get_state_full();
    assert_eq!(
        state.upcoming.iter().map(|t| t.id).collect::<Vec<_>>(),
        vec![101]
    );
}

#[test]
fn test_remove_upcoming_after_respects_shuffle_play_order() {
    let queue = QueueManager::new();
    // tracks: 101..105 at indices 0..4. Shuffle play order = [2,0,4,1,3]
    // -> 103(current), then upcoming 101, 105, 102, 104.
    queue.set_queue_with_order(
        (101..=105).map(create_test_track).collect(),
        Some(2),
        true,
        Some(vec![2, 0, 4, 1, 3]),
    );

    // Keep upcoming positions 0..=1 (101, 105); drop 2, 3 (102, 104).
    let removed = queue.remove_upcoming_after(1);

    assert_eq!(removed, 2);
    let state = queue.get_state_full();
    assert_eq!(state.current_track.map(|t| t.id), Some(103));
    assert_eq!(
        state.upcoming.iter().map(|t| t.id).collect::<Vec<_>>(),
        vec![101, 105]
    );
}

#[test]
fn test_remove_upcoming_after_invalidates_marker_in_removed_range() {
    let queue = QueueManager::new();
    queue.set_queue((101..=105).map(create_test_track).collect(), Some(0));
    queue.set_stop_after(104); // upcoming position 2 -> removed by after(1)

    queue.remove_upcoming_after(1);

    assert_eq!(queue.get_stop_after(), None);
}
