//! "Artists from the same place" location-based scene discovery.
//! Ports `v2_discover_artists_by_location`'s core pipeline (the scene
//! cache + progress events are omitted; subdivision resolution and
//! affinity scoring are kept).
//!
//! The pipeline (area resolution -> genre selection -> MB search/score ->
//! Qobuz validation) is a single sequential algorithm; the genre-selection
//! and candidate-scoring/-validation steps have been split into sibling
//! files (`discovery_location_genres.rs`, `discovery_location_candidates.rs`)
//! purely to fit the file line budget, and are called in order below.

use qbz_integrations::musicbrainz::genre::genre_summary;
use qbz_integrations::musicbrainz::{AffinitySeeds, LocationDiscoveryResponse};
use qbz_models::FrontendAdapter;

use crate::error::CoreError;

use super::super::QbzCore;
use super::discovery_location_genres::{rank_candidates, select_search_genres};

impl<A: FrontendAdapter + Send + Sync + 'static> QbzCore<A> {
    /// "Artists from the same place" — given a source artist's MBID,
    /// area and genre/tag seeds, find other artists from that area
    /// who share the genres, validated against Qobuz. Ports
    /// v2_discover_artists_by_location's core pipeline (the scene
    /// cache + progress events are omitted; subdivision resolution
    /// and affinity scoring are kept).
    #[allow(clippy::too_many_arguments)]
    pub async fn discover_artists_by_location(
        &self,
        source_mbid: &str,
        area_id: Option<&str>,
        area_name: &str,
        country: Option<&str>,
        genres: Vec<String>,
        tags: Vec<String>,
        limit: usize,
        offset: usize,
    ) -> Result<LocationDiscoveryResponse, CoreError> {
        // Step 0: smart area resolution — city → parent subdivision
        // for broader results (Leyton → England, Seattle →
        // Washington).
        let (search_name, display_name) = match area_id {
            Some(aid) => match self.musicbrainz.resolve_parent_subdivision(aid).await {
                Ok(Some((subdivision, _))) => {
                    let display = country
                        .map(|c| format!("{}, {}", c, subdivision))
                        .unwrap_or_else(|| subdivision.clone());
                    (subdivision, display)
                }
                _ => {
                    let display = country
                        .map(|c| format!("{}, {}", c, area_name))
                        .unwrap_or_else(|| area_name.to_string());
                    (area_name.to_string(), display)
                }
            },
            None => {
                let display = country
                    .map(|c| format!("{}, {}", c, area_name))
                    .unwrap_or_else(|| area_name.to_string());
                (area_name.to_string(), display)
            }
        };

        let source_seeds = AffinitySeeds {
            genres: genres.clone(),
            tags: tags.clone(),
            normalized_seeds: genres.iter().chain(tags.iter()).cloned().collect(),
        };

        let search_genres = select_search_genres(&genres, &tags);
        if search_genres.is_empty() {
            return Ok(LocationDiscoveryResponse {
                artists: Vec::new(),
                scene_label: format!("{} scene", display_name),
                genre_summary: String::new(),
                total_candidates: 0,
                has_more: false,
                next_offset: 0,
            });
        }

        let candidate_map = self
            .collect_location_candidates(
                &search_genres,
                &search_name,
                area_id,
                country,
                source_mbid,
                &source_seeds,
            )
            .await;

        let scored = rank_candidates(candidate_map);
        let total_candidates = scored.len();
        let to_validate: Vec<_> = scored.into_iter().skip(offset).take(limit).collect();

        let validated = self.validate_location_candidates(&to_validate).await;

        let scene_label = country
            .map(|c| c.to_string())
            .unwrap_or_else(|| display_name.clone());
        let next_offset = offset + to_validate.len();
        Ok(LocationDiscoveryResponse {
            artists: validated,
            scene_label,
            genre_summary: genre_summary(&source_seeds),
            total_candidates,
            has_more: next_offset < total_candidates,
            next_offset,
        })
    }
}
