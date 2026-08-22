//! Validating that a name resolves to a real Qobuz artist with a compatible genre.

use super::name_match::normalize_name;
use super::SuggestionsEngine;
use qbz_qobuz::QobuzClient;

impl SuggestionsEngine {
    /// Validate that an artist exists in Qobuz with their own catalog AND compatible genre
    ///
    /// Returns Some((artist_id, artist_name)) if found, None otherwise.
    /// This prevents false matches for:
    /// - Session musicians without their own page (e.g., "Martin Lopez" drummer)
    /// - Names that match different artists (e.g., Latin "Martin Mendez" vs bassist)
    /// - Artists with incompatible genres (bachata/merengue artist vs metal drummer)
    pub(super) async fn validate_qobuz_artist(
        &self,
        client: &QobuzClient,
        name: &str,
    ) -> Option<(u64, String)> {
        // Normalize name for comparison (removes accents: å→a, é→e, etc.)
        let name_normalized = normalize_name(name);

        // Search Qobuz for artist - try original name first
        let mut results = match client.search_artists(name, 10, 0, None).await {
            Ok(r) => r,
            Err(e) => {
                log::warn!(
                    "[SuggestionsEngine] Artist search failed for '{}': {}",
                    name,
                    e
                );
                return None;
            }
        };

        // If no results and name has accents, also try normalized name
        // e.g., "Mikael Åkerfeldt" -> "Mikael Akerfeldt"
        if results.items.is_empty() && name != name_normalized {
            log::debug!(
                "[SuggestionsEngine] No results for '{}', trying normalized '{}'",
                name,
                name_normalized
            );
            if let Ok(r) = client.search_artists(&name_normalized, 10, 0, None).await {
                results = r;
            }
        }

        // Look for exact name match (comparing normalized versions)
        let mut candidate: Option<(u64, String)> = None;

        for artist in &results.items {
            let artist_normalized = normalize_name(&artist.name);

            // Exact match (after accent normalization)
            // This allows "Mikael Åkerfeldt" to match "Mikael Akerfeldt"
            if artist_normalized == name_normalized && artist.albums_count.unwrap_or(0) > 0 {
                candidate = Some((artist.id, artist.name.clone()));
                break;
            }
        }

        // Also try "The X" variant (e.g., "Beatles" -> "The Beatles")
        if candidate.is_none() {
            let the_name_normalized = format!("the {}", name_normalized);
            for artist in &results.items {
                let artist_normalized = normalize_name(&artist.name);
                if artist_normalized == the_name_normalized && artist.albums_count.unwrap_or(0) > 0
                {
                    candidate = Some((artist.id, artist.name.clone()));
                    break;
                }
            }
        }

        // If we found a candidate, verify their genre is compatible
        if let Some((artist_id, artist_name)) = candidate {
            if self
                .has_incompatible_genre(client, artist_id, &artist_name)
                .await
            {
                log::info!(
                    "[SuggestionsEngine] Rejecting '{}' (ID: {}) - incompatible genre detected",
                    artist_name,
                    artist_id
                );
                return None;
            }
            return Some((artist_id, artist_name));
        }

        None
    }
}
