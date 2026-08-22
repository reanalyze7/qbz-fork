//! Pure helper functions for [`discover_artists_by_location`]: picking which
//! genres/tags to search MusicBrainz with, and turning the raw per-artist
//! score accumulator into a sorted candidate list. Split out of
//! `discovery_location.rs` to stay under the file line budget — these two
//! steps have no `&self` dependency, so they live as free functions.
//!
//! [`discover_artists_by_location`]: super::QbzCore::discover_artists_by_location

use std::collections::HashMap;

use qbz_integrations::musicbrainz::genre::{extract_affinity_seeds, is_broad_genre};
use qbz_integrations::musicbrainz::Tag;

/// Step 2: pick search genres, dropping overly broad tags that would return
/// the whole country's catalog. Falls back to the raw (unfiltered) list if
/// everything was filtered out as "broad".
pub(super) fn select_search_genres(genres: &[String], tags: &[String]) -> Vec<String> {
    let mut search_genres: Vec<String> = if genres.is_empty() {
        tags.iter()
            .filter(|s| !is_broad_genre(s))
            .take(3)
            .cloned()
            .collect()
    } else {
        genres
            .iter()
            .chain(tags.iter().take(2))
            .filter(|s| !is_broad_genre(s))
            .cloned()
            .collect()
    };
    if search_genres.is_empty() {
        // Everything was broad — fall back to the raw list.
        search_genres = if genres.is_empty() {
            tags.iter().take(3).cloned().collect()
        } else {
            genres.iter().take(3).cloned().collect()
        };
    }
    search_genres
}

/// Step 3: score + sort candidates. Final score = affinity + (genre_hits-1)*15.
pub(super) fn rank_candidates(
    candidate_map: HashMap<String, (String, i32, usize, Vec<String>)>,
) -> Vec<(String, String, Vec<String>, i32)> {
    let mut scored: Vec<(String, String, Vec<String>, i32)> = candidate_map
        .into_iter()
        .map(|(mbid, (name, score, genre_hits, tag_list))| {
            let candidate_seeds = extract_affinity_seeds(
                &tag_list
                    .iter()
                    .map(|name| Tag {
                        name: name.clone(),
                        count: Some(1),
                    })
                    .collect::<Vec<_>>(),
            );
            let multi_genre_bonus = ((genre_hits as i32) - 1) * 15;
            (mbid, name, candidate_seeds.genres, score + multi_genre_bonus)
        })
        .collect();
    scored.sort_by(|a, b| b.3.cmp(&a.3).then_with(|| a.0.cmp(&b.0)));
    scored
}
