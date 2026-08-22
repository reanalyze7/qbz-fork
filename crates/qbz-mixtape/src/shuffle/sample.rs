//! Album-capped hybrid sampling: round-robin draw from randomized buckets.

use qbz_models::QueueTrack as CoreQueueTrack;
use rand::RngExt;
use std::collections::BTreeMap;

use super::{ALBUM_CAP_MIN, ALBUM_CAP_PCT};

/// Picks up to `requested` tracks from `tracks` such that no album contributes
/// more than `cap = max(ALBUM_CAP_MIN, ceil(requested * ALBUM_CAP_PCT))` picks.
/// Albums are drawn round-robin in a per-round randomized order; within each
/// album, the surviving picks are themselves shuffled.
///
/// May return fewer than `requested` tracks if the per-album cap and bucket
/// sizes do not add up to `requested`.
pub fn hybrid_sample<R: rand::Rng>(
    tracks: Vec<CoreQueueTrack>,
    requested: usize,
    rng: &mut R,
) -> Vec<CoreQueueTrack> {
    if tracks.is_empty() || requested == 0 {
        return Vec::new();
    }

    let cap =
        (((requested as f32) * ALBUM_CAP_PCT).ceil() as usize).max(ALBUM_CAP_MIN);

    // Bucket original indices by album_id (None goes to a synthetic bucket).
    let mut by_album: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (i, t) in tracks.iter().enumerate() {
        let key = t
            .album_id
            .clone()
            .unwrap_or_else(|| "_unknown".to_string());
        by_album.entry(key).or_default().push(i);
    }

    // Shuffle each bucket and apply quota = min(cap, bucket.len()) by truncating.
    let mut buckets: Vec<Vec<usize>> = by_album
        .into_values()
        .map(|mut indices| {
            fisher_yates(&mut indices, rng);
            indices.truncate(cap);
            indices
        })
        .collect();

    // Round-robin pick with album order permuted each round. `pop` from each
    // bucket — order within a bucket is already random, so pop_back is fine.
    let mut picked: Vec<usize> = Vec::with_capacity(requested);
    while picked.len() < requested {
        fisher_yates(&mut buckets, rng);
        let mut progress = false;
        for bucket in buckets.iter_mut() {
            if let Some(idx) = bucket.pop() {
                picked.push(idx);
                progress = true;
                if picked.len() >= requested {
                    break;
                }
            }
        }
        if !progress {
            break;
        }
    }

    // Map indices back to tracks, preserving the picked order.
    use std::collections::HashMap;
    let mut by_index: HashMap<usize, CoreQueueTrack> =
        tracks.into_iter().enumerate().collect();
    picked
        .into_iter()
        .filter_map(|i| by_index.remove(&i))
        .collect()
}

pub(super) fn fisher_yates<T, R: rand::Rng>(slice: &mut [T], rng: &mut R) {
    if slice.len() < 2 {
        return;
    }
    for i in (1..slice.len()).rev() {
        let j = rng.random_range(0..=i);
        slice.swap(i, j);
    }
}
