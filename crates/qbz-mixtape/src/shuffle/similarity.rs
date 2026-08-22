//! Token-set similarity + union-find grouping by (artist, similar title).

use qbz_models::QueueTrack as CoreQueueTrack;
use std::collections::BTreeMap;

use super::normalize::{normalize_artist, normalize_title};
use super::SIMILARITY_THRESHOLD;

/// Token-set similarity in `[0.0, 1.0]`, modeled on RapidFuzz's
/// `token_set_ratio`. Inputs are expected to be already normalized.
pub fn token_set_ratio(a: &str, b: &str) -> f32 {
    use std::collections::BTreeSet;

    let tokens_a: BTreeSet<&str> = a.split_whitespace().collect();
    let tokens_b: BTreeSet<&str> = b.split_whitespace().collect();

    if tokens_a.is_empty() && tokens_b.is_empty() {
        return 1.0;
    }
    if tokens_a.is_empty() || tokens_b.is_empty() {
        return 0.0;
    }

    let inter: Vec<&str> = tokens_a.intersection(&tokens_b).copied().collect();
    let diff_a: Vec<&str> = tokens_a.difference(&tokens_b).copied().collect();
    let diff_b: Vec<&str> = tokens_b.difference(&tokens_a).copied().collect();

    let t1 = inter.join(" ");
    let t2 = join_with_intersection(&t1, &diff_a);
    let t3 = join_with_intersection(&t1, &diff_b);

    let r12 = strsim::normalized_levenshtein(&t1, &t2);
    let r13 = strsim::normalized_levenshtein(&t1, &t3);
    let r23 = strsim::normalized_levenshtein(&t2, &t3);

    r12.max(r13).max(r23) as f32
}

pub(super) fn join_with_intersection(t1: &str, diff: &[&str]) -> String {
    if diff.is_empty() {
        return t1.to_string();
    }
    let diff_joined = diff.join(" ");
    if t1.is_empty() {
        diff_joined
    } else {
        format!("{} {}", t1, diff_joined)
    }
}

/// For each track index, returns the index of the group representative under
/// the artist-bucketed token-set similarity grouping. Tracks in different
/// artist buckets always end up in different groups.
pub(super) fn build_similarity_groups(tracks: &[CoreQueueTrack]) -> Vec<usize> {
    let n = tracks.len();
    let mut parent: Vec<usize> = (0..n).collect();

    // Bucket indices by normalized artist.
    let mut by_artist: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (i, t) in tracks.iter().enumerate() {
        by_artist
            .entry(normalize_artist(&t.artist))
            .or_default()
            .push(i);
    }

    // Within each artist bucket, union pairs whose normalized titles are
    // similar enough.
    for indices in by_artist.values() {
        let titles: Vec<String> = indices
            .iter()
            .map(|&i| normalize_title(&tracks[i].title))
            .collect();
        for a in 0..indices.len() {
            for b in (a + 1)..indices.len() {
                if token_set_ratio(&titles[a], &titles[b]) >= SIMILARITY_THRESHOLD {
                    uf_union(&mut parent, indices[a], indices[b]);
                }
            }
        }
    }

    (0..n).map(|i| uf_find(&mut parent, i)).collect()
}

pub(super) fn uf_find(parent: &mut [usize], mut x: usize) -> usize {
    while parent[x] != x {
        parent[x] = parent[parent[x]]; // path compression (halving)
        x = parent[x];
    }
    x
}

pub(super) fn uf_union(parent: &mut [usize], a: usize, b: usize) {
    let ra = uf_find(parent, a);
    let rb = uf_find(parent, b);
    if ra != rb {
        // Smaller index becomes parent so behavior is order-stable for tests
        // that don't shuffle.
        let (root, child) = if ra < rb { (ra, rb) } else { (rb, ra) };
        parent[child] = root;
    }
}
