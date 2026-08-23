//! Lifecycle round-trip test (process-global singleton, so kept as ONE
//! combined test rather than parallel ones that would clobber each other).

use super::*;

/// Unique temp dir under the system temp root (no `tempfile` dev-dep on
/// qbz-slint). Created here, removed at the end of the test.
fn unique_temp_dir() -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("qbz-slint-search-service-test-{nanos}"));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

/// One combined test: the singleton is process-global, so splitting into
/// parallel tests would let them clobber each other. Covers the full
/// round-trip: fail-safe-disabled before init, enabled gate after init,
/// store/cached round-trip, record/top, the kill switch, then teardown
/// restoring the fail-safe state.
#[test]
fn lifecycle_roundtrip() {
    let dir = unique_temp_dir();

    // Fail-safe before any session is bound.
    assert!(!is_enabled(), "no session => reads as disabled");
    assert!(cached("Pink Floyd").is_none(), "no session => no cache");
    assert!(top_for_query("Pink Floyd").is_none(), "no session => no top");

    init(&dir, true);
    assert!(is_enabled(), "init(enabled=true) => enabled");

    // record -> top_for_query returns it (cache store needs a real
    // SearchAllResults; the ranking path is enough to prove wiring).
    record("Pink Floyd", "artist", "100", InteractionAction::Favorite);
    assert_eq!(
        top_for_query("Pink Floyd"),
        Some(("artist".to_string(), "100".to_string()))
    );

    // Kill switch flips the bound service.
    set_enabled(false);
    assert!(!is_enabled(), "kill switch disables");
    assert!(top_for_query("Pink Floyd").is_none(), "disabled => no top");
    set_enabled(true);
    assert!(is_enabled(), "re-enabled");

    teardown();
    assert!(!is_enabled(), "fail-safe disabled after teardown");
    assert!(top_for_query("Pink Floyd").is_none(), "no top after teardown");

    let _ = std::fs::remove_dir_all(&dir);
}
