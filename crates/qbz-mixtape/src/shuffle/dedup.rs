//! Similarity-based dedup over a `Vec<CoreQueueTrack>`.

use qbz_models::QueueTrack as CoreQueueTrack;
use rand::RngExt;
use std::collections::{BTreeMap, BTreeSet};

use super::similarity::build_similarity_groups;

/// Count of distinct songs in `tracks` after similarity-based grouping.
/// Deterministic — does not use an RNG. Same input always yields the same
/// count.
pub fn unique_track_count(tracks: &[CoreQueueTrack]) -> usize {
    if tracks.is_empty() {
        return 0;
    }
    let groups = build_similarity_groups(tracks);
    groups.iter().copied().collect::<BTreeSet<usize>>().len()
}

/// Removes near-duplicate tracks from `tracks`. Two tracks are considered the
/// same song when their normalized artists match exactly AND their normalized
/// titles score at or above [`super::SIMILARITY_THRESHOLD`]. From each duplicate
/// group, one survivor is picked at random via `rng`.
///
/// The output preserves the original input order of the surviving tracks.
pub fn dedup_by_similarity<R: rand::Rng>(
    tracks: Vec<CoreQueueTrack>,
    rng: &mut R,
) -> Vec<CoreQueueTrack> {
    if tracks.is_empty() {
        return tracks;
    }

    let groups = build_similarity_groups(&tracks);

    // Bucket original indices by their group representative.
    let mut by_group: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (i, &g) in groups.iter().enumerate() {
        by_group.entry(g).or_default().push(i);
    }

    // Pick one random survivor index per group.
    let mut survivors: BTreeSet<usize> = BTreeSet::new();
    for indices in by_group.values() {
        let chosen = if indices.len() == 1 {
            indices[0]
        } else {
            indices[rng.random_range(0..indices.len())]
        };
        survivors.insert(chosen);
    }

    tracks
        .into_iter()
        .enumerate()
        .filter(|(i, _)| survivors.contains(i))
        .map(|(_, t)| t)
        .collect()
}
