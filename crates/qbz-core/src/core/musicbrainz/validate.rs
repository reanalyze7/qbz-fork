//! Qobuz-side validation of MusicBrainz discovery candidates. Split out
//! of `discovery.rs` for line budget; `pub(crate)` because
//! `discovery_location.rs` (a sibling module) also calls this for the
//! location-based discovery pipeline.

use std::collections::HashSet;

use qbz_integrations::musicbrainz::DiscoveryArtist;
use qbz_models::FrontendAdapter;

use super::super::helpers::normalize_artist_name;
use super::super::QbzCore;

impl<A: FrontendAdapter + Send + Sync + 'static> QbzCore<A> {
    /// Validate MusicBrainz candidates against Qobuz (exact normalized-name
    /// match).
    pub(crate) async fn validate_discovery_on_qobuz(
        &self,
        candidates: &[(String, String)],
        max: usize,
        known_ids: &HashSet<u64>,
    ) -> Vec<DiscoveryArtist> {
        let mut out: Vec<DiscoveryArtist> = Vec::new();
        for (mbid, name) in candidates {
            if out.len() >= max {
                break;
            }
            let Ok(page) = self.search_artists(name, 1, 0, None).await else {
                continue;
            };
            let Some(first) = page.items.first() else {
                continue;
            };
            if normalize_artist_name(&first.name) != normalize_artist_name(name) {
                continue;
            }
            // Tauri's `!local_known_qobuz_ids.contains(&artist.id)`
            // gate — never suggest an artist the user has already
            // listened to >2 times.
            if known_ids.contains(&first.id) {
                continue;
            }
            out.push(DiscoveryArtist {
                mbid: mbid.clone(),
                name: first.name.clone(),
                qobuz_id: Some(first.id),
            });
        }
        out
    }
}
