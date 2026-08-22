//! `search_artist_tracks_with_limit`: validates the artist exists in Qobuz
//! before searching, to avoid false matches for session musicians.
//!
//! Locking discipline: the `store` guard (used only to resolve a name) is
//! scoped to its own block and dropped before any `.await`. The
//! `qobuz_client` read guard is a tokio lock and is intentionally held across
//! the `validate_qobuz_artist`/`search_tracks` awaits, matching the
//! pre-split behavior.

use super::super::name_match::names_similar;
use super::super::{SuggestedTrack, SuggestionsEngine};

impl SuggestionsEngine {
    /// Search Qobuz for tracks by an artist with custom limit
    ///
    /// First validates that the artist EXISTS in Qobuz (has a dedicated artist page).
    /// This prevents false matches for session musicians who don't have their own catalog
    /// (e.g., "Martin Lopez" drummer returning tracks from unrelated "Martin Lopez" artists).
    pub(in super::super) async fn search_artist_tracks_with_limit(
        &self,
        artist_mbid: &str,
        artist_name: Option<&str>,
        similarity: f32,
        limit: usize,
    ) -> Vec<SuggestedTrack> {
        let search_query = match artist_name {
            Some(name) => name.to_string(),
            None => {
                // Try to get name from store
                let guard__ = self.store.lock().await;
                if let Some(store) = guard__.as_ref() {
                    store
                        .get_artist_name(artist_mbid)
                        .unwrap_or_else(|| artist_mbid.to_string())
                } else {
                    artist_mbid.to_string()
                }
            }
        };

        let guard__ = self.qobuz_client.read().await;
        let Some(client) = guard__.as_ref() else {
            log::warn!("[SuggestionsEngine] No active Qobuz session; skipping");
            return Vec::new();
        };

        // Step 1: Validate artist exists in Qobuz with their own catalog
        // This prevents searching for session musicians who don't have artist pages
        let validated_artist = self.validate_qobuz_artist(&client, &search_query).await;

        if validated_artist.is_none() {
            log::info!(
                "[SuggestionsEngine] Skipping '{}' - no Qobuz artist page found or incompatible genre",
                search_query
            );
            return Vec::new();
        }

        let (qobuz_artist_id, qobuz_artist_name) = validated_artist.unwrap();
        log::info!(
            "[SuggestionsEngine] Validated '{}' -> Qobuz artist '{}' (ID: {})",
            search_query,
            qobuz_artist_name,
            qobuz_artist_id
        );

        // Step 2: Search for tracks by artist name
        // Fetch many more since search results include tracks where the artist appears,
        // not just tracks BY the artist. We filter down to exact matches.
        let search_limit = ((limit * 5) as u32).max(100).min(500); // Between 100-500
        match client
            .search_tracks(&search_query, search_limit, 0, None)
            .await
        {
            Ok(results) => {
                let mut tracks = Vec::new();

                for item in results.items {
                    // Verify the track's performer matches the validated Qobuz artist
                    // Use both ID matching (best) and name matching (fallback)
                    let performer_id = item.performer.as_ref().map(|p| p.id);
                    let performer_name = item
                        .performer
                        .as_ref()
                        .map(|p| p.name.clone())
                        .unwrap_or_default();

                    // Prefer ID match (exact), fall back to name comparison
                    let is_match = performer_id == Some(qobuz_artist_id)
                        || names_similar(&performer_name, &qobuz_artist_name);

                    if is_match {
                        tracks.push(self.track_to_suggested(&item, artist_mbid, similarity));
                        if tracks.len() >= limit {
                            break;
                        }
                    }
                }

                tracks
            }
            Err(e) => {
                log::warn!("Failed to search tracks for {}: {}", search_query, e);
                Vec::new()
            }
        }
    }
}
