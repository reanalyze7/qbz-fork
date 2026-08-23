//! Process-global singleton test: kept as ONE combined test rather than
//! several parallel `#[test]` fns, since parallel tests would clobber the
//! shared `SERVICE` static.

use super::*;

/// Unique temp dir under the system temp root (no `tempfile` dev-dep on
/// qbz-slint). Created here, removed at the end of the test.
fn unique_temp_dir() -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("qbz-slint-pinned-test-{nanos}"));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn item(kind: &str, id: &str, title: &str) -> PinnedItem {
    PinnedItem {
        kind: kind.to_string(),
        id: id.to_string(),
        title: title.to_string(),
        subtitle: String::new(),
        artwork_url: String::new(),
        pinned_at: 0, // ignored on write; the service stamps now
    }
}

/// One combined test: the singleton is process-global, so splitting into
/// parallel tests would let them clobber each other. Covers the full
/// round-trip: empty state after init, pin reflected in check + list +
/// snapshot, unpin, then teardown restores the fail-open state.
#[test]
fn lifecycle_roundtrip() {
    let dir = unique_temp_dir();

    init_for_user(&dir);
    assert!(!is_pinned("album", "abc"), "nothing pinned yet");
    assert!(list().is_empty(), "fresh store lists nothing");
    assert!(keys_snapshot().is_empty(), "fresh snapshot is empty");
    assert_eq!(count(), 0);

    pin(&item("album", "abc", "An Album")).expect("pin succeeds with a bound store");
    assert!(is_pinned("album", "abc"), "pinned item is pinned");
    assert!(!is_pinned("playlist", "abc"), "kinds are isolated");
    assert_eq!(count(), 1);
    assert!(
        keys_snapshot().contains(&("album".to_string(), "abc".to_string())),
        "snapshot contains the pinned key"
    );
    let all = list();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].title, "An Album");

    unpin("album", "abc").expect("unpin succeeds");
    assert!(!is_pinned("album", "abc"), "unpinned item is gone");
    unpin("album", "nope").expect("absent unpin is Ok");
    assert_eq!(count(), 0);

    // Rebind persists across sessions: pin, teardown, re-init, still pinned.
    pin(&item("playlist", "7", "P")).expect("pin");
    teardown();
    assert!(!is_pinned("playlist", "7"), "fail-open after teardown");
    assert!(list().is_empty(), "empty list after teardown");
    assert!(keys_snapshot().is_empty(), "empty snapshot after teardown");
    assert_eq!(count(), 0);
    assert!(
        pin(&item("album", "x", "X")).is_err(),
        "mutation with no session returns the error string"
    );
    assert!(
        unpin("album", "x").is_err(),
        "unpin with no session returns the error string"
    );
    init_for_user(&dir);
    assert!(is_pinned("playlist", "7"), "pin persisted across rebind");

    teardown();
    let _ = std::fs::remove_dir_all(&dir);
}
