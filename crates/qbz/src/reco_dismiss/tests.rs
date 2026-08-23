use super::*;

/// Unique temp dir under the system temp root (no `tempfile` dev-dep on
/// qbz-slint). Created here, removed at the end of the test.
fn unique_temp_dir() -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("qbz-slint-reco-dismiss-test-{nanos}"));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

/// One combined test: the path singleton is process-global, so splitting
/// into parallel tests would let them clobber each other. Covers the full
/// lifecycle: bind, dismiss (idempotent + snapshot backfill), persistence
/// across a re-bind, remove, unknown-format tolerance, and the fail-open
/// unbound state.
#[test]
fn lifecycle_roundtrip() {
    let dir = unique_temp_dir();

    init_for_user(&dir);
    assert!(ids_snapshot().is_empty(), "fresh store has no ids");
    assert!(list().is_empty(), "fresh store has no rows");

    dismiss(42, "Artist X", "https://img/x.jpg");
    dismiss(7, "Artist Y", "");
    assert!(ids_snapshot().contains(&42));
    assert!(ids_snapshot().contains(&7));
    assert_eq!(list().len(), 2);

    // Idempotent: a re-dismiss does not duplicate, but backfills an empty
    // snapshot field.
    dismiss(42, "Artist X", "https://img/x.jpg");
    dismiss(7, "Artist Y", "https://img/y.jpg");
    assert_eq!(list().len(), 2, "no duplicate rows");
    assert_eq!(list()[1].image_url, "https://img/y.jpg", "image backfilled");

    // Id 0 is rejected and never matches.
    dismiss(0, "Nobody", "");
    assert!(!ids_snapshot().contains(&0));

    // Persistence: re-binding the same dir loads the file back.
    teardown();
    assert!(ids_snapshot().is_empty(), "fail-open after teardown");
    dismiss(9, "Lost", ""); // no-op while unbound; must not panic
    init_for_user(&dir);
    assert!(ids_snapshot().contains(&42), "rows survive a re-bind");
    assert!(!ids_snapshot().contains(&9), "unbound mutation did not persist");

    // Undo.
    remove(42);
    assert!(!ids_snapshot().contains(&42));
    assert_eq!(list().len(), 1);
    remove(42); // absent: no-op, no write needed

    // Unknown-format tolerance: junk in the file reads as an empty store.
    std::fs::write(dir.join(FILE_NAME), b"{ not json !!").expect("write junk");
    assert!(ids_snapshot().is_empty(), "corrupt file fails open");
    assert!(list().is_empty());
    // A partially-unknown row shape degrades to defaults, not a crash.
    std::fs::write(
        dir.join(FILE_NAME),
        br#"{"artists":[{"artist_id":5},{"artist_id":0,"name":"zero"}]}"#,
    )
    .expect("write partial");
    assert!(ids_snapshot().contains(&5), "partial row still loads");
    assert!(!ids_snapshot().contains(&0), "id 0 row is dropped");

    teardown();
    let _ = std::fs::remove_dir_all(&dir);
}
