//! Searching Qobuz for tracks by a given artist.

mod by_qobuz_id;
mod with_limit;

use super::{SuggestedTrack, SuggestionsEngine};

impl SuggestionsEngine {
    /// Search Qobuz for tracks by an artist (uses default tracks_per_artist limit)
    pub(super) async fn search_artist_tracks(
        &self,
        artist_mbid: &str,
        artist_name: Option<&str>,
        similarity: f32,
    ) -> Vec<SuggestedTrack> {
        self.search_artist_tracks_with_limit(
            artist_mbid,
            artist_name,
            similarity,
            self.config.tracks_per_artist,
        )
        .await
    }
}
