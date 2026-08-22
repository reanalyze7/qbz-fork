use super::mk_track;
use crate::shuffle::unique_track_count;

// ──────── unique_track_count ────────

#[test]
fn unique_track_count_zero_for_empty() {
    assert_eq!(unique_track_count(&[]), 0);
}

#[test]
fn unique_track_count_distinct_tracks() {
    let tracks = vec![
        mk_track(1, "Yesterday", "The Beatles", Some("a1")),
        mk_track(2, "Hey Jude", "The Beatles", Some("a1")),
        mk_track(3, "Let It Be", "The Beatles", Some("a1")),
    ];
    assert_eq!(unique_track_count(&tracks), 3);
}

#[test]
fn unique_track_count_groups_versions() {
    let tracks = vec![
        mk_track(1, "Yesterday", "The Beatles", Some("a1")),
        mk_track(2, "Yesterday (Live)", "The Beatles", Some("a2")),
        mk_track(3, "Yesterday - 2003 Remaster", "The Beatles", Some("a3")),
        mk_track(4, "Hey Jude", "The Beatles", Some("a1")),
        mk_track(5, "Let It Be", "The Beatles", Some("a1")),
    ];
    // 3 versions of "Yesterday" collapse to 1, plus 2 distinct → 3 unique.
    assert_eq!(unique_track_count(&tracks), 3);
}

#[test]
fn unique_track_count_respects_artist_buckets() {
    let tracks = vec![
        mk_track(1, "Yesterday", "The Beatles", Some("a1")),
        mk_track(2, "Yesterday", "Boyz II Men", Some("a2")),
    ];
    // Same title, different artists → not deduplicated.
    assert_eq!(unique_track_count(&tracks), 2);
}

#[test]
fn unique_track_count_is_deterministic() {
    let tracks = vec![
        mk_track(1, "Yesterday", "The Beatles", Some("a1")),
        mk_track(2, "Yesterday (Live)", "The Beatles", Some("a2")),
        mk_track(3, "Hey Jude", "The Beatles", Some("a1")),
    ];
    let c1 = unique_track_count(&tracks);
    let c2 = unique_track_count(&tracks);
    let c3 = unique_track_count(&tracks);
    assert_eq!(c1, c2);
    assert_eq!(c2, c3);
    assert_eq!(c1, 2);
}
