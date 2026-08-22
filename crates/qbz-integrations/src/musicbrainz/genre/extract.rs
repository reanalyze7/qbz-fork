//! Turning raw MusicBrainz tags into affinity seeds and display summaries.

use std::collections::HashSet;

use crate::musicbrainz::{AffinitySeeds, Tag};

use super::normalize::{is_noisy_tag, normalize_genre};
use super::tables::{GENRE_MIN_VOTES, MAX_GENRES, MAX_TAGS};

/// Extract affinity seeds from MusicBrainz tags.
///
/// Genres (primary signal): top-voted tags with enough votes, normalized.
/// Tags (secondary signal): remaining useful tags after noise filtering.
pub fn extract_affinity_seeds(tags: &[Tag]) -> AffinitySeeds {
    if tags.is_empty() {
        return AffinitySeeds {
            genres: Vec::new(),
            tags: Vec::new(),
            normalized_seeds: Vec::new(),
        };
    }

    // Sort by vote count descending
    let mut sorted_tags: Vec<_> = tags.iter().collect();
    sorted_tags.sort_by(|a, b| b.count.unwrap_or(0).cmp(&a.count.unwrap_or(0)));

    let mut genres = Vec::new();
    let mut secondary_tags = Vec::new();
    let mut seen_normalized = HashSet::new();

    for tag in &sorted_tags {
        let count = tag.count.unwrap_or(0);
        if count < GENRE_MIN_VOTES {
            continue;
        }

        if is_noisy_tag(&tag.name) {
            continue;
        }

        let normalized = normalize_genre(&tag.name);

        if seen_normalized.contains(&normalized) {
            continue;
        }
        seen_normalized.insert(normalized.clone());

        if genres.len() < MAX_GENRES {
            genres.push(normalized);
        } else if secondary_tags.len() < MAX_TAGS {
            secondary_tags.push(normalized);
        }
    }

    let normalized_seeds: Vec<String> = genres
        .iter()
        .chain(secondary_tags.iter())
        .cloned()
        .collect();

    AffinitySeeds {
        genres,
        tags: secondary_tags,
        normalized_seeds,
    }
}

/// Compute the genre summary string for display (e.g., "grunge / alternative rock")
pub fn genre_summary(seeds: &AffinitySeeds) -> String {
    if seeds.genres.is_empty() {
        return String::new();
    }
    seeds
        .genres
        .iter()
        .take(3)
        .cloned()
        .collect::<Vec<_>>()
        .join(" / ")
}
