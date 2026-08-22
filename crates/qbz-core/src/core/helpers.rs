//! Free pure helpers with no `QbzCore` dependency: name normalization,
//! a seeded deterministic shuffle, and playlist-duplicate set math.

use qbz_models::PlaylistDuplicateResult;

/// Normalize an artist name for dedupe: trim, lowercase, collapse
/// whitespace. Used by the discovery pipeline so "Iron  Maiden" and
/// "iron maiden" hash to the same key in the dismiss store.
pub fn normalize_artist_name(name: &str) -> String {
    name.trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Deterministic shuffle keyed by `seed_mbid` (and optionally a tag).
/// Same artist page produces the same order across runs; different
/// artist or different fallback tag produces a different order.
/// `pub(crate)` because both `musicbrainz::discovery` and
/// `musicbrainz::discovery_location` call it.
pub(crate) fn shuffle_with_seed<T>(items: &mut Vec<T>, seed_mbid: &str, secondary_tag: Option<&str>) {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    seed_mbid.hash(&mut hasher);
    if let Some(t) = secondary_tag {
        t.hash(&mut hasher);
    }
    let mut h = hasher.finish();
    // Fisher-Yates with a simple xorshift PRNG seeded from the hash —
    // keeps qbz-core free of the rand dep.
    let n = items.len();
    for i in (1..n).rev() {
        h ^= h << 13;
        h ^= h >> 7;
        h ^= h << 17;
        let j = (h % ((i + 1) as u64)) as usize;
        items.swap(i, j);
    }
}

/// Pure set-intersection behind [`super::QbzCore::check_playlist_duplicates`] —
/// split out so the duplicate logic is unit-testable without a live Qobuz
/// client. `existing` = the playlist's current track ids; `track_ids` = the
/// ids the user wants to add. Returns the Tauri-shaped result (total checked,
/// how many are already present, and the set of those duplicate ids).
pub(crate) fn compute_playlist_duplicates(
    existing: &[u64],
    track_ids: &[u64],
) -> PlaylistDuplicateResult {
    let existing_set: std::collections::HashSet<u64> = existing.iter().copied().collect();
    let duplicate_track_ids: std::collections::HashSet<u64> = track_ids
        .iter()
        .copied()
        .filter(|id| existing_set.contains(id))
        .collect();
    PlaylistDuplicateResult {
        total_tracks: track_ids.len(),
        duplicate_count: duplicate_track_ids.len(),
        duplicate_track_ids,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_playlist_duplicates_intersects_input_with_existing() {
        // Existing playlist has 10, 20, 30. Adding 20, 30, 40, 50:
        // 20 and 30 are duplicates; 40 and 50 are new.
        let existing = [10u64, 20, 30];
        let to_add = [20u64, 30, 40, 50];
        let r = compute_playlist_duplicates(&existing, &to_add);
        assert_eq!(r.total_tracks, 4);
        assert_eq!(r.duplicate_count, 2);
        assert!(r.duplicate_track_ids.contains(&20));
        assert!(r.duplicate_track_ids.contains(&30));
        assert!(!r.duplicate_track_ids.contains(&40));
    }

    #[test]
    fn compute_playlist_duplicates_none_when_disjoint() {
        let r = compute_playlist_duplicates(&[1u64, 2, 3], &[4u64, 5]);
        assert_eq!(r.total_tracks, 2);
        assert_eq!(r.duplicate_count, 0);
        assert!(r.duplicate_track_ids.is_empty());
    }

    #[test]
    fn compute_playlist_duplicates_empty_input() {
        let r = compute_playlist_duplicates(&[1u64, 2, 3], &[]);
        assert_eq!(r.total_tracks, 0);
        assert_eq!(r.duplicate_count, 0);
    }
}
