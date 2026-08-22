//! `generate_suggestions`: the main pipeline, orchestrating steps 1-7.
//!
//! Each numbered step is delegated to a private helper method in a sibling
//! file (kept here as one async fn only for the parts that must see the
//! whole shared state: `all_tracks` / `source_artists` are threaded through
//! explicitly as `&mut` parameters rather than fields, so no lock is ever
//! implicated in the threading). Locking discipline per step is documented
//! in each step's own file.

mod ensure_vectors;
mod finalize;
mod qobuz_similar_fallback;
mod related_artists;
mod search_playlist_artists;
mod search_related_artists;

use super::{SuggestedTrack, SuggestionResult, SuggestionsEngine};
use std::collections::HashSet;
use std::time::Instant;

impl SuggestionsEngine {
    /// Generate suggestions for a playlist
    ///
    /// # Arguments
    /// * `playlist_artists` - Artist info (MBID, name) from the playlist
    /// * `exclude_track_ids` - Track IDs to exclude (already in playlist)
    /// * `include_reasons` - Whether to include reason strings (dev mode)
    pub async fn generate_suggestions(
        &self,
        playlist_artists: &[(String, String)], // (mbid, name)
        exclude_track_ids: &HashSet<u64>,
        include_reasons: bool,
    ) -> Result<SuggestionResult, String> {
        if playlist_artists.is_empty() {
            log::debug!("[SuggestionsEngine] Empty playlist, returning empty");
            return Ok(SuggestionResult {
                tracks: Vec::new(),
                source_artists: Vec::new(),
                playlist_artists_count: 0,
                similar_artists_count: 0,
            });
        }

        // Extract MBIDs for vector operations
        let playlist_artist_mbids: Vec<String> = playlist_artists
            .iter()
            .map(|(mbid, _)| mbid.clone())
            .collect();

        // 1. Ensure vectors exist for playlist artists (skip if configured)
        self.ensure_playlist_vectors(playlist_artists).await;

        // 2. Compute combined playlist vector
        log::debug!("[SuggestionsEngine] Step 2: Computing playlist vector");
        let step2_start = Instant::now();
        let playlist_vector = self.compute_playlist_vector(&playlist_artist_mbids).await?;
        log::debug!(
            "[SuggestionsEngine] Step 2 completed in {:?}, vector empty={}",
            step2_start.elapsed(),
            playlist_vector.is_empty()
        );

        if playlist_vector.is_empty() {
            log::warn!("[SuggestionsEngine] Playlist vector is empty, returning empty result");
            return Ok(SuggestionResult {
                tracks: Vec::new(),
                source_artists: Vec::new(),
                playlist_artists_count: playlist_artist_mbids.len(),
                similar_artists_count: 0,
            });
        }

        // 3. Find related artists (using direct relationships, not vector similarity)
        log::debug!("[SuggestionsEngine] Step 3: Finding related artists");
        let step3_start = Instant::now();
        let similar_artists = self.find_related_artists(&playlist_artist_mbids).await?;
        log::debug!(
            "[SuggestionsEngine] Step 3 completed in {:?}, found {} related artists",
            step3_start.elapsed(),
            similar_artists.len()
        );

        let similar_artists_count = similar_artists.len();
        let mut source_artists = Vec::new();
        let mut all_tracks: Vec<SuggestedTrack> = Vec::new();

        // 4a. First, search for tracks by playlist artists themselves (highest relevance)
        self.search_playlist_artist_tracks(
            playlist_artists,
            exclude_track_ids,
            include_reasons,
            &mut source_artists,
            &mut all_tracks,
        )
        .await;

        // 4b. Then search for tracks by related/similar artists
        self.search_related_artist_tracks(
            &similar_artists,
            exclude_track_ids,
            include_reasons,
            &playlist_artist_mbids,
            &mut source_artists,
            &mut all_tracks,
        )
        .await;

        // 4c. If pool is still small, use Qobuz's "similar artists" API as fallback
        // This gives us artists that definitely exist in Qobuz
        self.qobuz_similar_fallback(
            playlist_artists,
            exclude_track_ids,
            include_reasons,
            &mut source_artists,
            &mut all_tracks,
        )
        .await?;

        // 5-7. Deduplicate, shuffle, and truncate the pool to its final form.
        let tracks = self.finalize_track_pool(all_tracks);

        Ok(SuggestionResult {
            tracks,
            source_artists,
            playlist_artists_count: playlist_artist_mbids.len(),
            similar_artists_count,
        })
    }
}
