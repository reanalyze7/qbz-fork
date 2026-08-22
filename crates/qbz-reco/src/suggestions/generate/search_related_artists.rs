//! Step 4b: search Qobuz for tracks by related/similar artists found in step 3.

use super::super::{SuggestedTrack, SuggestionsEngine};
use crate::store::SimilarArtist;
use std::collections::HashSet;
use std::time::Instant;

impl SuggestionsEngine {
    pub(super) async fn search_related_artist_tracks(
        &self,
        similar_artists: &[SimilarArtist],
        exclude_track_ids: &HashSet<u64>,
        include_reasons: bool,
        playlist_artist_mbids: &[String],
        source_artists: &mut Vec<String>,
        all_tracks: &mut Vec<SuggestedTrack>,
    ) {
        log::debug!(
            "[SuggestionsEngine] Step 4b: Searching tracks for {} related artists",
            similar_artists.len()
        );
        let step4b_start = Instant::now();

        for (i, artist) in similar_artists.iter().enumerate() {
            if artist.similarity < self.config.min_similarity {
                continue;
            }

            if let Some(name) = &artist.name {
                if !source_artists.contains(name) {
                    source_artists.push(name.clone());
                }
            }

            // Search Qobuz for tracks by this related artist
            let tracks = self
                .search_artist_tracks(&artist.mbid, artist.name.as_deref(), artist.similarity)
                .await;

            for mut track in tracks {
                // Skip if already in playlist
                if exclude_track_ids.contains(&track.track_id) {
                    continue;
                }

                // Add reason if requested
                if include_reasons {
                    track.reason = Some(self.generate_reason(
                        &artist.mbid,
                        artist.name.as_deref(),
                        artist.similarity,
                        playlist_artist_mbids,
                    ));
                }

                all_tracks.push(track);
            }

            // Stop if we have enough tracks
            if all_tracks.len() >= self.config.max_pool_size * 2 {
                log::debug!(
                    "[SuggestionsEngine] Reached extended pool size {} after {} related artists",
                    all_tracks.len(),
                    i + 1
                );
                break;
            }
        }
        log::debug!(
            "[SuggestionsEngine] Step 4b completed in {:?}, got {} total tracks",
            step4b_start.elapsed(),
            all_tracks.len()
        );
    }
}
