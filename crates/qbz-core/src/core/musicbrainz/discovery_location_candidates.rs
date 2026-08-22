//! `&self`-dependent steps of [`discover_artists_by_location`]: the MB
//! tag+area candidate search/scoring pass, and the final Qobuz validation
//! pass. Split out of `discovery_location.rs` to stay under the file line
//! budget.
//!
//! [`discover_artists_by_location`]: super::QbzCore::discover_artists_by_location

use std::collections::HashMap;

use qbz_integrations::musicbrainz::location::compute_affinity_score;
use qbz_integrations::musicbrainz::{AffinitySeeds, LocationCandidate};
use qbz_models::FrontendAdapter;

use super::super::QbzCore;

impl<A: FrontendAdapter + Send + Sync + 'static> QbzCore<A> {
    /// Step 2/3: MB tag+area search per genre, dedupe + score.
    /// Returns mbid -> (name, score_sum, genre_hits, tags).
    #[allow(clippy::too_many_arguments)]
    pub(super) async fn collect_location_candidates(
        &self,
        search_genres: &[String],
        search_name: &str,
        area_id: Option<&str>,
        country: Option<&str>,
        source_mbid: &str,
        source_seeds: &AffinitySeeds,
    ) -> HashMap<String, (String, i32, usize, Vec<String>)> {
        let mut candidate_map: HashMap<String, (String, i32, usize, Vec<String>)> = HashMap::new();
        let per_genre_limit = 200usize;
        for genre in search_genres {
            let result = self
                .musicbrainz
                .search_artists_by_tag_and_area(genre, search_name, country, per_genre_limit, 0)
                .await;
            let Ok(response) = result else {
                continue;
            };
            for artist in &response.artists {
                if artist.id == source_mbid {
                    continue;
                }
                let candidate_tags: Vec<String> = artist
                    .tags
                    .as_ref()
                    .map(|list| {
                        list.iter()
                            .filter(|t| t.count.unwrap_or(0) > 0)
                            .map(|t| t.name.clone())
                            .collect()
                    })
                    .unwrap_or_default();
                let same_city = artist
                    .begin_area
                    .as_ref()
                    .map(|ba| {
                        ba.name.eq_ignore_ascii_case(search_name)
                            || area_id.map(|aid| ba.id == aid).unwrap_or(false)
                    })
                    .unwrap_or(false);
                let same_country = artist
                    .area
                    .as_ref()
                    .map(|a| a.name.eq_ignore_ascii_case(search_name))
                    .unwrap_or(false);
                let score =
                    compute_affinity_score(&candidate_tags, source_seeds, same_city, same_country);
                let entry = candidate_map
                    .entry(artist.id.clone())
                    .or_insert_with(|| (artist.name.clone(), 0, 0, Vec::new()));
                entry.1 += score;
                entry.2 += 1;
                for tag in &candidate_tags {
                    if !entry.3.contains(tag) {
                        entry.3.push(tag.clone());
                    }
                }
            }
        }
        candidate_map
    }

    /// Step 4: validate ranked candidates against Qobuz (exact
    /// normalized-name match, pick the one with the most albums as a
    /// popularity proxy).
    pub(super) async fn validate_location_candidates(
        &self,
        to_validate: &[(String, String, Vec<String>, i32)],
    ) -> Vec<LocationCandidate> {
        let mut validated: Vec<LocationCandidate> = Vec::new();
        for (mbid, mb_name, candidate_genres, score) in to_validate {
            let Ok(results) = self.search_artists(mb_name, 5, 0, None).await else {
                continue;
            };
            let mb_norm = super::super::helpers::normalize_artist_name(mb_name);
            let best = results
                .items
                .iter()
                .filter(|a| super::super::helpers::normalize_artist_name(&a.name) == mb_norm)
                .max_by_key(|a| a.albums_count.unwrap_or(0));
            if let Some(qobuz_artist) = best {
                let image_url = qobuz_artist
                    .image
                    .as_ref()
                    .and_then(|img| img.small.as_ref().or(img.thumbnail.as_ref()).cloned());
                validated.push(LocationCandidate {
                    mbid: mbid.clone(),
                    mb_name: mb_name.clone(),
                    qobuz_id: Some(qobuz_artist.id as i64),
                    qobuz_name: Some(qobuz_artist.name.clone()),
                    qobuz_image: image_url,
                    score: *score,
                    genres: candidate_genres.clone(),
                    qobuz_albums_count: qobuz_artist.albums_count,
                });
            }
        }
        validated
    }
}
