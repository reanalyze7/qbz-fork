//! Searching Qobuz tracks by a known Qobuz artist ID (used by the Qobuz
//! similar-artists fallback, where the caller already holds the client).

use super::super::{SuggestedTrack, SuggestionsEngine};
use qbz_qobuz::QobuzClient;

impl SuggestionsEngine {
    /// Search Qobuz for tracks by Qobuz artist ID (more reliable when we already validated the artist)
    /// Takes client reference to avoid deadlock when caller already holds the lock
    pub(in super::super) async fn search_artist_tracks_by_qobuz_id(
        &self,
        client: &QobuzClient,
        qobuz_artist_id: u64,
        artist_name: &str,
        similarity: f32,
    ) -> Vec<SuggestedTrack> {
        let limit = self.config.tracks_per_artist;

        // Search by artist name but verify tracks belong to this specific Qobuz artist ID
        match client
            .search_tracks(artist_name, (limit * 3) as u32, 0, None)
            .await
        {
            Ok(results) => {
                let mut tracks = Vec::new();

                for item in results.items {
                    // Only accept tracks from this exact artist (by ID)
                    let performer_id = item.performer.as_ref().map(|p| p.id);
                    if performer_id != Some(qobuz_artist_id) {
                        continue;
                    }

                    tracks.push(self.track_to_suggested_with_qobuz_id(
                        &item,
                        qobuz_artist_id,
                        similarity,
                    ));
                    if tracks.len() >= limit {
                        break;
                    }
                }

                tracks
            }
            Err(e) => {
                log::warn!(
                    "Failed to search tracks for {} (Qobuz ID {}): {}",
                    artist_name,
                    qobuz_artist_id,
                    e
                );
                Vec::new()
            }
        }
    }
}
