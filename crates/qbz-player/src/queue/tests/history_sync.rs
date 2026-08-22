use super::create_test_track;
use super::QueueManager;

// ===================================================================
// LANE C DIAGNOSTIC TESTS (B1 — sync_current_to_id pointer robustness)
// These reproduce concrete states where the QueueManager pointer ends
// up at a DIFFERENT track than the audio engine is actually playing.
// ===================================================================

/// B1 ROOT-CAUSE (a): live track id NOT present in `state.tracks`.
/// When the audio engine has gaplessly advanced into a track that is no
/// longer in the queue (e.g. the queue was replaced by a brand-new list
/// while the OLD track was still draining, or an infinite-radio / single-
/// track-replace), `sync_current_to_id` returns None and the pointer is
/// left untouched — pointing at the STALE old track. Both NPB and sidebar
/// then read the wrong `current_track`, and the poll loop never retries
/// (it updates `last_track_id` and `continue`s). This is the "cleared the
/// queue and it did NOT fix it" symptom.
#[test]
fn test_sync_current_to_id_unknown_id_leaves_pointer_stale() {
    let queue = QueueManager::new();
    queue.add_track(create_test_track(100)); // "Enjoy the Silence"
    queue.add_track(create_test_track(101));
    queue.play_index(0); // pointer = 0 -> track 100

    // The audio engine is actually playing track 999 ("Beetlebum"), which
    // is NOT in the queue (queue was replaced / radio handoff).
    let result = queue.sync_current_to_id(999);

    // Sync fails — pointer cannot be corrected.
    assert!(result.is_none(), "sync should return None for unknown id");

    // The pointer is STILL on the stale track 100, so the now-playing
    // surfaces report the WRONG track while audio plays 999.
    let state = queue.get_state_full();
    assert_eq!(
        state.current_track.unwrap().id,
        100,
        "pointer stays stale at the OLD track when sync_current_to_id can't find the live id"
    );
}

/// B1 ROOT-CAUSE (a'): clear(keep_current) when the kept track is NOT the
/// audible one. The user clears the queue to "fix" the stale now-playing,
/// but clear keeps the track at the (already stale) `current_index`, so the
/// wrong track is now the SOLE entry — and because its id != the live audio
/// id, a subsequent sync_current_to_id STILL can't correct it. Reproduces
/// the user's "cleared the queue and it did NOT fix it".
#[test]
fn test_clear_keep_current_preserves_the_wrong_track() {
    let queue = QueueManager::new();
    queue.add_track(create_test_track(100)); // stale "now playing"
    queue.add_track(create_test_track(101));
    queue.play_index(0); // pointer stuck on 100 (audio actually plays 999)

    queue.clear(true); // user hits "Clear Queue" to fix the stale row

    let state = queue.get_state_full();
    // The kept track is the WRONG one (100), not the audible 999.
    assert_eq!(state.current_track.unwrap().id, 100);

    // And sync still can't fix it: 999 isn't in the 1-track queue.
    assert!(queue.sync_current_to_id(999).is_none());
    assert_eq!(queue.get_state_full().current_track.unwrap().id, 100);
}

/// B1 ROOT-CAUSE (b): DUPLICATE track ids in the queue.
/// `sync_current_to_id` uses `position(|t| t.id == id)`, which returns the
/// FIRST match. If the same track id appears twice (very common: a track
/// added to a playlist twice, an album with a repeated bonus track, or a
/// QConnect queue echo), advancing the audio into the SECOND occurrence
/// resyncs the pointer to the FIRST occurrence — wrong index. The
/// now-playing track id is "right" but the upcoming list, position, and any
/// index-derived state are computed from the wrong slot.
#[test]
fn test_sync_current_to_id_duplicate_id_resyncs_to_first_occurrence() {
    let queue = QueueManager::new();
    queue.add_track(create_test_track(100)); // idx 0
    queue.add_track(create_test_track(200)); // idx 1
    queue.add_track(create_test_track(100)); // idx 2  (DUPLICATE of idx 0)
    queue.add_track(create_test_track(300)); // idx 3
    queue.play_index(1); // pointer = 1 (track 200)

    // Audio gaplessly advanced into the SECOND copy of track 100 (idx 2).
    let (_, moved) = queue
        .sync_current_to_id(100)
        .expect("id 100 is present, so sync succeeds");
    assert!(moved, "pointer moved off track 200");

    let state = queue.get_state_full();
    // current_index resynced to the FIRST occurrence (idx 0), NOT idx 2.
    assert_eq!(
        state.current_index,
        Some(0),
        "sync_current_to_id resyncs to the FIRST id match (idx 0), not the actually-playing idx 2"
    );
    // Consequence: the UP-NEXT list is computed from the wrong slot. From
    // idx 0 the upcoming is [200, 100, 300]; from the real idx 2 it should
    // be [300]. The sidebar shows a wrong upcoming list.
    let upcoming_ids: Vec<u64> = state.upcoming.iter().map(|t| t.id).collect();
    assert_eq!(
        upcoming_ids,
        vec![200, 100, 300],
        "upcoming is derived from the wrong (first-occurrence) index"
    );
}

#[test]
fn play_index_to_same_track_does_not_push_history() {
    let queue = QueueManager::new();
    queue.add_track(create_test_track(1));
    queue.add_track(create_test_track(2));
    queue.play_index(0); // current None -> 0, no push

    // Re-align to the SAME index — the QConnect controller materialize path
    // calls play_index with the index already current. It must NOT record a
    // spurious "previous" entry (the track-shows-twice-in-History bug).
    queue.play_index(0);
    assert!(
        queue.get_state().history.is_empty(),
        "re-aligning to the current track must not add a history entry"
    );

    // A genuine move still records the outgoing track.
    queue.play_index(1);
    let history = queue.get_state().history;
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].id, 1);
}
