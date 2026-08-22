use super::create_test_track;
use super::queue_with_played_history;

#[test]
fn test_set_queue_with_order_clears_history_when_tracks_completely_different() {
    let queue = queue_with_played_history(5, 3);
    assert_eq!(queue.get_state().history.len(), 3);

    // No overlap with the previous queue; history must drop entirely.
    let fresh = vec![
        create_test_track(100),
        create_test_track(101),
        create_test_track(102),
    ];
    queue.set_queue_with_order(fresh, Some(0), false, None);

    let after = queue.get_state();
    assert!(after.history.is_empty());
}

#[test]
fn test_set_queue_preserves_history_on_pure_reorder() {
    // Mirror test for set_queue (non-with-order variant).
    let queue = queue_with_played_history(5, 3);
    let before = queue.get_state();
    assert_eq!(
        before.history.iter().map(|t| t.id).collect::<Vec<_>>(),
        vec![3, 2, 1]
    );

    let reordered = vec![
        create_test_track(5),
        create_test_track(4),
        create_test_track(3),
        create_test_track(2),
        create_test_track(1),
    ];
    queue.set_queue(reordered, Some(1)); // current track 4 now at idx 1

    let after = queue.get_state();
    assert_eq!(
        after.history.iter().map(|t| t.id).collect::<Vec<_>>(),
        vec![3, 2, 1]
    );
}

#[test]
fn test_set_queue_with_order_remaps_history_indices_after_reorder() {
    // Verify that after a reorder, the internal indices stored in history
    // actually point to the right new tracks (not just that ids match
    // through the get_state() projection accidentally).
    let queue = queue_with_played_history(4, 3);

    // Reverse order. Old tracks 1,2,3,4 -> new tracks 4,3,2,1.
    // Old history: indices [0, 1, 2] -> ids [1, 2, 3].
    // New mapping: id=1->idx 3, id=2->idx 2, id=3->idx 1.
    // Expected new history indices: [3, 2, 1] (front-to-back).
    let reversed = vec![
        create_test_track(4),
        create_test_track(3),
        create_test_track(2),
        create_test_track(1),
    ];
    queue.set_queue_with_order(reversed, Some(0), false, None);

    // Inspect internal state to verify the indices, not just rendered ids.
    let state = queue.state.lock().unwrap();
    assert_eq!(
        state.history.iter().copied().collect::<Vec<_>>(),
        vec![3, 2, 1]
    );
}
