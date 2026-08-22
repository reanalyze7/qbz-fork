//! Secondary-tag fallback for `musicbrainz_discover_artists` — split out
//! of `discovery.rs` for line budget. Only reached when the primary tag
//! didn't return enough validated results.

use std::collections::HashSet;

use qbz_models::FrontendAdapter;

use super::super::helpers::{normalize_artist_name, shuffle_with_seed};
use super::super::QbzCore;

impl<A: FrontendAdapter + Send + Sync + 'static> QbzCore<A> {
    /// Search the secondary tag, dedupe against the primary results and
    /// the dismiss/known-artist filters, and append validated top-ups
    /// (in place) to `results` up to `max_results`.
    #[allow(clippy::too_many_arguments)]
    pub(super) async fn discover_secondary_tag_fallback(
        &self,
        seed_mbid: &str,
        seed_norm: &str,
        similar_norm: &HashSet<String>,
        dismissed_primary: &HashSet<String>,
        secondary_tag: &str,
        dismissed_per_tag: &(dyn Fn(&str) -> HashSet<String> + Send + Sync),
        known_ids: &HashSet<u64>,
        known_names: &HashSet<String>,
        max_results: usize,
        results: &mut Vec<qbz_integrations::musicbrainz::DiscoveryArtist>,
    ) {
        let dismissed_secondary = dismissed_per_tag(&secondary_tag.to_lowercase());
        let existing_mbids: HashSet<String> = results.iter().map(|r| r.mbid.clone()).collect();
        let Ok(secondary) = self
            .musicbrainz
            .search_artists_by_tag(secondary_tag, 30)
            .await
        else {
            return;
        };

        let mut secondary_candidates: Vec<(String, String)> = Vec::new();
        for a in &secondary.artists {
            let n = normalize_artist_name(&a.name);
            if n == seed_norm
                || a.id.eq_ignore_ascii_case(seed_mbid)
                || similar_norm.contains(&n)
                || dismissed_primary.contains(&n)
                || dismissed_secondary.contains(&n)
                || known_names.contains(&n)
                || existing_mbids.contains(&a.id)
            {
                continue;
            }
            secondary_candidates.push((a.id.clone(), a.name.clone()));
        }
        shuffle_with_seed(&mut secondary_candidates, seed_mbid, Some(secondary_tag));
        let remaining = max_results.saturating_sub(results.len());
        let mut more = self
            .validate_discovery_on_qobuz(&secondary_candidates, remaining, known_ids)
            .await;
        results.append(&mut more);
    }
}
