use super::*;

/// Unique temp dir under the system temp root (no `tempfile` dev-dep on
/// qbz-slint). Created here, removed at the end of the test.
fn unique_temp_dir() -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("qbz-slint-blacklist-test-{nanos}"));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

/// One combined test: the singleton is process-global, so splitting into
/// parallel tests would let them clobber each other. Covers the full
/// round-trip: empty snapshot + default-enabled after init, add reflected in
/// both check + snapshot, then teardown restores the fail-open state.
#[test]
fn lifecycle_roundtrip() {
    let dir = unique_temp_dir();

    init_for_user(&dir);
    assert!(ids_snapshot().is_empty(), "fresh store has no ids");
    assert!(is_enabled(), "blacklist defaults to enabled");
    assert!(!is_blacklisted(42), "nothing blacklisted yet");

    add(42, "X", None).expect("add succeeds with a bound store");
    assert!(is_blacklisted(42), "added id is blacklisted");
    assert!(is_blacklisted_id_str("42"), "string-id check matches");
    assert!(ids_snapshot().contains(&42), "snapshot contains the added id");
    assert_eq!(count(), 1);

    // Album axis: orthogonal, String-keyed, shares the enabled flag.
    assert!(!is_album_blacklisted("zzz"), "nothing album-blocked yet");
    add_album("zzz", "Bogus", "X", "", None).expect("add_album succeeds");
    assert!(is_album_blacklisted("zzz"), "added album id is blocked");
    assert!(!is_album_blacklisted(""), "empty album id never matches");
    assert!(
        album_ids_snapshot().contains("zzz"),
        "album snapshot contains the added id"
    );
    assert_eq!(album_count(), 1);
    // A blocked album drops via the shared stamp predicate, artist-independent.
    assert!(stamp_row("qobuz", &[], Some("zzz")), "album-blocked row drops");
    assert!(
        !stamp_row("local", &[], Some("zzz")),
        "non-qobuz row is protected even when album-blocked"
    );
    assert_eq!(count(), 1, "album add did not touch the artist count");

    teardown();
    assert!(!is_blacklisted(42), "fail-open after teardown");
    assert!(ids_snapshot().is_empty(), "empty snapshot after teardown");
    assert!(!is_album_blacklisted("zzz"), "album fail-open after teardown");
    assert!(album_ids_snapshot().is_empty(), "empty album snapshot");
    assert_eq!(count(), 0);
    assert_eq!(album_count(), 0);
    assert!(is_enabled(), "default-enabled after teardown");
    assert!(
        add(1, "Y", None).is_err(),
        "mutation with no session returns the Tauri error string"
    );
    assert!(
        add_album("a", "b", "c", "", None).is_err(),
        "album mutation with no session returns the error string"
    );

    let _ = std::fs::remove_dir_all(&dir);
}
