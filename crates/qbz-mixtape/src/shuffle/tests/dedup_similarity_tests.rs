use super::{deterministic_rng, mk_track};
use crate::shuffle::dedup_by_similarity;
use rand::SeedableRng;
use std::collections::BTreeSet;

// ──────── dedup_by_similarity ────────

#[test]
fn dedup_empty_returns_empty() {
    let mut rng = deterministic_rng();
    assert!(dedup_by_similarity(vec![], &mut rng).is_empty());
}

#[test]
fn dedup_single_track_passes_through() {
    let mut rng = deterministic_rng();
    let out = dedup_by_similarity(
        vec![mk_track(1, "Yesterday", "The Beatles", Some("a1"))],
        &mut rng,
    );
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].id, 1);
}

#[test]
fn dedup_keeps_distinct_titles() {
    let mut rng = deterministic_rng();
    let tracks = vec![
        mk_track(1, "Yesterday", "The Beatles", Some("a1")),
        mk_track(2, "Hey Jude", "The Beatles", Some("a1")),
        mk_track(3, "Let It Be", "The Beatles", Some("a1")),
        mk_track(4, "Help!", "The Beatles", Some("a1")),
        mk_track(5, "Imagine", "The Beatles", Some("a1")),
    ];
    let out = dedup_by_similarity(tracks, &mut rng);
    assert_eq!(out.len(), 5);
}

#[test]
fn dedup_collapses_versions() {
    let mut rng = deterministic_rng();
    let tracks = vec![
        mk_track(1, "Yesterday", "The Beatles", Some("a1")),
        mk_track(2, "Yesterday (Live)", "The Beatles", Some("a2")),
        mk_track(3, "Yesterday - 2003 Remaster", "The Beatles", Some("a3")),
    ];
    let out = dedup_by_similarity(tracks, &mut rng);
    assert_eq!(out.len(), 1);
    // The survivor must be one of the three input ids.
    assert!([1u64, 2, 3].contains(&out[0].id));
}

#[test]
fn dedup_respects_artist_buckets() {
    let mut rng = deterministic_rng();
    let tracks = vec![
        mk_track(1, "Yesterday", "The Beatles", Some("a1")),
        mk_track(2, "Yesterday", "Boyz II Men", Some("a2")),
    ];
    let out = dedup_by_similarity(tracks, &mut rng);
    assert_eq!(out.len(), 2);
}

#[test]
fn dedup_random_winner_varies_across_seeds() {
    // Over many seeds, all three versions should be selected at least once.
    let mut seen: BTreeSet<u64> = BTreeSet::new();
    for seed in 0..200u64 {
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let tracks = vec![
            mk_track(1, "Yesterday", "The Beatles", Some("a1")),
            mk_track(2, "Yesterday (Live)", "The Beatles", Some("a2")),
            mk_track(3, "Yesterday - 2003 Remaster", "The Beatles", Some("a3")),
        ];
        let out = dedup_by_similarity(tracks, &mut rng);
        assert_eq!(out.len(), 1);
        seen.insert(out[0].id);
    }
    assert_eq!(seen.len(), 3, "over 200 seeds, all 3 versions should win at least once; got {:?}", seen);
}

#[test]
fn dedup_preserves_input_order_of_survivors() {
    let mut rng = deterministic_rng();
    let tracks = vec![
        mk_track(10, "Hey Jude", "The Beatles", Some("a1")),
        mk_track(20, "Yesterday", "The Beatles", Some("a1")),
        mk_track(30, "Let It Be", "The Beatles", Some("a1")),
    ];
    let out = dedup_by_similarity(tracks, &mut rng);
    // None of these collapse, so all survive in original order.
    assert_eq!(out.iter().map(|t| t.id).collect::<Vec<_>>(), vec![10, 20, 30]);
}
