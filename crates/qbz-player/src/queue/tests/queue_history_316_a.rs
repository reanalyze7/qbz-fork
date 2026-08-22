use super::create_test_track;
use super::queue_with_played_history;

// --- Bug #316 history-preservation regression tests ---

#[test]
fn test_set_queue_with_order_preserves_history_on_pure_reorder() {
    // Played 3 tracks, current is on track 4 (id=4).
    let queue = queue_with_played_history(5, 3);
    let before = queue.get_state();
    assert_eq!(
        before.history.iter().map(|t| t.id).collect::<Vec<_>>(),
        vec![3, 2, 1]
    );

    // Same tracks, completely reordered. Current track (id=4) at new index 0.
    let reordered = vec![
        create_test_track(4),
        create_test_track(2),
        create_test_track(5),
        create_test_track(1),
        create_test_track(3),
    ];
    queue.set_queue_with_order(reordered, Some(0), false, None);

    let after = queue.get_state();
    // History rendered newest-first; ids must survive the reorder identically.
    assert_eq!(
        after.history.iter().map(|t| t.id).collect::<Vec<_>>(),
        vec![3, 2, 1]
    );
}

#[test]
fn test_set_queue_with_order_preserves_history_when_tracks_added() {
    // Played track 1, then 2; current is track 3.
    let queue = queue_with_played_history(3, 2);

    // Same tracks plus 2 new ones (4, 5). Current still on track 3 (new index 2).
    let expanded = vec![
        create_test_track(1),
        create_test_track(2),
        create_test_track(3),
        create_test_track(4),
        create_test_track(5),
    ];
    queue.set_queue_with_order(expanded, Some(2), false, None);

    let after = queue.get_state();
    assert_eq!(
        after.history.iter().map(|t| t.id).collect::<Vec<_>>(),
        vec![2, 1]
    );
}

#[test]
fn test_set_queue_with_order_drops_only_removed_tracks_from_history() {
    // Played tracks 1, 2, 3; current on track 4 (id=4).
    let queue = queue_with_played_history(5, 3);
    let before = queue.get_state();
    assert_eq!(
        before.history.iter().map(|t| t.id).collect::<Vec<_>>(),
        vec![3, 2, 1]
    );

    // Remove track id=2 from queue; tracks 1 and 3 survive in history.
    let trimmed = vec![
        create_test_track(1),
        create_test_track(3),
        create_test_track(4),
        create_test_track(5),
    ];
    queue.set_queue_with_order(trimmed, Some(2), false, None);

    let after = queue.get_state();
    assert_eq!(
        after.history.iter().map(|t| t.id).collect::<Vec<_>>(),
        vec![3, 1]
    );
}
