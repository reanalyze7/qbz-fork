//! Step 1: ensure playlist-artist vectors exist and are fresh (or skip).

use super::super::SuggestionsEngine;
use std::time::Instant;

impl SuggestionsEngine {
    /// Ensure vectors exist for all playlist artists (unless configured to skip).
    pub(super) async fn ensure_playlist_vectors(&self, playlist_artists: &[(String, String)]) {
        let step1_start = Instant::now();
        if self.config.skip_vector_build {
            log::debug!("[SuggestionsEngine] Step 1: SKIPPED (skip_vector_build=true), using only cached vectors");
            return;
        }

        log::debug!(
            "[SuggestionsEngine] Step 1: Ensuring vectors for {} artists",
            playlist_artists.len()
        );
        for (i, (mbid, name)) in playlist_artists.iter().enumerate() {
            let artist_start = Instant::now();
            let _ = self
                .builder
                .ensure_vector(mbid, Some(name), None, self.config.vector_max_age_days)
                .await;
            log::debug!(
                "[SuggestionsEngine] ensure_vector {}/{} took {:?}",
                i + 1,
                playlist_artists.len(),
                artist_start.elapsed()
            );
        }
        log::debug!(
            "[SuggestionsEngine] Step 1 completed in {:?}",
            step1_start.elapsed()
        );
    }
}
