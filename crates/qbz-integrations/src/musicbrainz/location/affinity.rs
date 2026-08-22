//! Genre/location affinity scoring for scene discovery candidates

use std::collections::HashSet;

use crate::musicbrainz::genre::normalize_genre;
use crate::musicbrainz::AffinitySeeds;

/// Affinity scoring weights
const SCORE_EXACT_CITY: i32 = 40;
const SCORE_SAME_COUNTRY: i32 = 15;
const SCORE_GENRE_CORE: i32 = 20;
const _SCORE_GENRE_SECONDARY: i32 = 10;
const SCORE_TAG_USEFUL: i32 = 8;
const SCORE_NOISY_ONLY: i32 = -12;

/// Compute affinity score for a candidate artist against the source seeds
pub fn compute_affinity_score(
    candidate_tags: &[String],
    source_seeds: &AffinitySeeds,
    same_city: bool,
    same_country: bool,
) -> i32 {
    let mut score: i32 = 0;

    if same_city {
        score += SCORE_EXACT_CITY;
    }
    if same_country {
        score += SCORE_SAME_COUNTRY;
    }

    // Normalize candidate tags for comparison
    let candidate_normalized: HashSet<String> = candidate_tags
        .iter()
        .map(|tag| normalize_genre(tag))
        .collect();

    // Core genre overlap
    let core_overlap = source_seeds
        .genres
        .iter()
        .filter(|g| candidate_normalized.contains(g.as_str()))
        .count();
    score += (core_overlap as i32) * SCORE_GENRE_CORE;

    // Secondary tag overlap
    let tag_overlap = source_seeds
        .tags
        .iter()
        .filter(|tag| candidate_normalized.contains(tag.as_str()))
        .count();
    score += (tag_overlap as i32) * SCORE_TAG_USEFUL;

    // Penalty: if candidate has tags but zero overlap with any seed
    if !candidate_normalized.is_empty() && core_overlap == 0 && tag_overlap == 0 {
        score += SCORE_NOISY_ONLY;
    }

    score
}

/// Build the scene cache key from location + seeds
pub fn build_scene_cache_key(area_id: &str, seeds: &AffinitySeeds) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    for seed in &seeds.normalized_seeds {
        seed.hash(&mut hasher);
    }
    let seed_hash = hasher.finish();

    format!("{}:{:x}", area_id, seed_hash)
}
