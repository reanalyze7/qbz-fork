//! Tag-based "You may also like" artist discovery.

use std::collections::HashSet;

use qbz_integrations::musicbrainz::DiscoveryResponse;
use qbz_models::FrontendAdapter;

use crate::error::CoreError;

use super::super::helpers::{normalize_artist_name, shuffle_with_seed};
use super::super::QbzCore;

impl<A: FrontendAdapter + Send + Sync + 'static> QbzCore<A> {
    /// "You may also like" tag-based discovery — finds artists that
    /// share the seed artist's primary genre tag on MusicBrainz, then
    /// validates exact name matches on Qobuz so the row can actually
    /// open the artist page. Filters out the seed itself, the artists
    /// already shown in the Similar section, and any dismissed
    /// artists (passed in by the caller; the dismiss store lives at
    /// the frontend layer).
    ///
    /// When the primary tag does not return enough validated results,
    /// the pipeline falls back to the secondary tag (see
    /// `discover_secondary_tag_fallback` in `discovery_fallback.rs`),
    /// dedupes against the primary's results, and tops up. Result
    /// ordering is deterministic per seed_mbid (same artist page =
    /// same shuffle).
    pub async fn musicbrainz_discover_artists(
        &self,
        seed_mbid: &str,
        seed_name: &str,
        similar_names: &[String],
        dismissed_per_tag: &(dyn Fn(&str) -> HashSet<String> + Send + Sync),
        known_artists: &(dyn Fn() -> (HashSet<u64>, HashSet<String>) + Send + Sync),
    ) -> Result<DiscoveryResponse, CoreError> {
        if !self.musicbrainz.is_enabled().await {
            return Ok(DiscoveryResponse {
                artists: Vec::new(),
                primary_tag: String::new(),
            });
        }

        let seed_tags = self
            .musicbrainz
            .get_artist_tags(seed_mbid)
            .await
            .unwrap_or_default();
        if seed_tags.is_empty() {
            return Ok(DiscoveryResponse {
                artists: Vec::new(),
                primary_tag: String::new(),
            });
        }

        let primary_tag = seed_tags[0].clone();
        let mb_results = self
            .musicbrainz
            .search_artists_by_tag(&primary_tag, 50)
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?;

        if mb_results.artists.is_empty() {
            return Ok(DiscoveryResponse {
                artists: Vec::new(),
                primary_tag,
            });
        }

        let seed_norm = normalize_artist_name(seed_name);
        let similar_norm: HashSet<String> =
            similar_names.iter().map(|n| normalize_artist_name(n)).collect();
        let dismissed_primary = dismissed_per_tag(&primary_tag.to_lowercase());
        let (known_ids, known_names) = known_artists();

        let mut candidates: Vec<(String, String)> = Vec::new();
        for artist in &mb_results.artists {
            let n = normalize_artist_name(&artist.name);
            if n == seed_norm
                || artist.id.eq_ignore_ascii_case(seed_mbid)
                || similar_norm.contains(&n)
                || dismissed_primary.contains(&n)
                || known_names.contains(&n)
            {
                continue;
            }
            candidates.push((artist.id.clone(), artist.name.clone()));
        }

        shuffle_with_seed(&mut candidates, seed_mbid, None);

        let max_results = 8;
        let min_results = 5;
        let mut results = self
            .validate_discovery_on_qobuz(&candidates, max_results, &known_ids)
            .await;

        if results.len() < min_results && seed_tags.len() > 1 {
            self.discover_secondary_tag_fallback(
                seed_mbid,
                &seed_norm,
                &similar_norm,
                &dismissed_primary,
                &seed_tags[1],
                dismissed_per_tag,
                &known_ids,
                &known_names,
                max_results,
                &mut results,
            )
            .await;
        }

        Ok(DiscoveryResponse {
            artists: results,
            primary_tag,
        })
    }
}
