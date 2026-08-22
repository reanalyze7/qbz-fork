use super::{deterministic_rng, mk_track};
use crate::shuffle::hybrid_sample;
use rand::SeedableRng;
use std::collections::{BTreeMap, BTreeSet};

// ──────── hybrid_sample ────────

#[test]
fn hybrid_sample_empty_returns_empty() {
    let mut rng = deterministic_rng();
    assert!(hybrid_sample(vec![], 10, &mut rng).is_empty());
}

#[test]
fn hybrid_sample_zero_requested_returns_empty() {
    let mut rng = deterministic_rng();
    let tracks = vec![mk_track(1, "A", "X", Some("a1"))];
    assert!(hybrid_sample(tracks, 0, &mut rng).is_empty());
}

#[test]
fn hybrid_sample_single_album_caps_at_floor() {
    // 1 album, 50 tracks, requested = 20.
    // cap = max(2, ceil(20 * 0.3)) = max(2, 6) = 6.
    // With only 1 album, total quota = 6, so result.len() = 6.
    let mut rng = deterministic_rng();
    let tracks: Vec<_> = (0..50)
        .map(|i| mk_track(i, &format!("Track {i}"), "X", Some("a1")))
        .collect();
    let out = hybrid_sample(tracks, 20, &mut rng);
    assert_eq!(out.len(), 6);
}

#[test]
fn hybrid_sample_returns_requested_when_distribution_allows() {
    // 10 albums × 20 tracks = 200; requested = 50.
    // cap = max(2, ceil(50 * 0.3)) = max(2, 15) = 15.
    // total quota = min(15, 20) * 10 = 150 > 50, so we hit `requested`.
    let mut rng = deterministic_rng();
    let mut tracks = Vec::new();
    for album in 0..10u64 {
        for track in 0..20u64 {
            tracks.push(mk_track(
                album * 100 + track,
                &format!("Track {album}-{track}"),
                "X",
                Some(&format!("a{album}")),
            ));
        }
    }
    let out = hybrid_sample(tracks, 50, &mut rng);
    assert_eq!(out.len(), 50);
}

#[test]
fn hybrid_sample_respects_album_cap_statistical() {
    // 3 albums × 100 tracks each; requested = 20; cap = max(2, 6) = 6.
    // Across 200 seeds, no album ever exceeds 6 picks.
    for seed in 0..200u64 {
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let mut tracks = Vec::new();
        for album in 0..3u64 {
            for track in 0..100u64 {
                tracks.push(mk_track(
                    album * 1000 + track,
                    &format!("T{album}-{track}"),
                    "X",
                    Some(&format!("alb{album}")),
                ));
            }
        }
        let out = hybrid_sample(tracks, 20, &mut rng);

        let mut per_album: BTreeMap<String, usize> = BTreeMap::new();
        for t in &out {
            let key = t.album_id.clone().unwrap();
            *per_album.entry(key).or_default() += 1;
        }
        let max_per = per_album.values().copied().max().unwrap_or(0);
        assert!(
            max_per <= 6,
            "seed {seed}: max picks per album was {max_per}, expected <= 6"
        );
    }
}

#[test]
fn hybrid_sample_distributes_across_albums() {
    // 5 albums × 50 tracks; requested = 15; cap = max(2, 5) = 5.
    // With round-robin, expect at least 3 albums represented.
    let mut rng = deterministic_rng();
    let mut tracks = Vec::new();
    for album in 0..5u64 {
        for track in 0..50u64 {
            tracks.push(mk_track(
                album * 1000 + track,
                &format!("T{album}-{track}"),
                "X",
                Some(&format!("alb{album}")),
            ));
        }
    }
    let out = hybrid_sample(tracks, 15, &mut rng);
    assert_eq!(out.len(), 15);
    let albums: BTreeSet<String> = out
        .iter()
        .map(|t| t.album_id.clone().unwrap())
        .collect();
    assert!(
        albums.len() >= 3,
        "expected >= 3 albums represented, got {} ({:?})",
        albums.len(),
        albums
    );
}

#[test]
fn hybrid_sample_groups_unknown_album_id() {
    // Tracks without album_id share the synthetic "_unknown" bucket and
    // therefore share one cap, not one per track.
    let mut rng = deterministic_rng();
    let tracks: Vec<_> = (0..50)
        .map(|i| mk_track(i, &format!("Track {i}"), "X", None))
        .collect();
    let out = hybrid_sample(tracks, 20, &mut rng);
    // Same as the single-album case: cap = 6.
    assert_eq!(out.len(), 6);
}
