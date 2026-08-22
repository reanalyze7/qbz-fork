//! Step 4a: search Qobuz for tracks by the playlist artists themselves
//! (highest relevance, similarity = 1.0).

use super::super::{SuggestedTrack, SuggestionsEngine};
use std::collections::HashSet;
use std::time::Instant;

impl SuggestionsEngine {
    pub(super) async fn search_playlist_artist_tracks(
        &self,
        playlist_artists: &[(String, String)],
        exclude_track_ids: &HashSet<u64>,
        include_reasons: bool,
        source_artists: &mut Vec<String>,
        all_tracks: &mut Vec<SuggestedTrack>,
    ) {
        log::info!(
            "[SuggestionsEngine] Step 4a: Searching tracks for {} playlist artists",
            playlist_artists.len()
        );
        let step4a_start = Instant::now();

        for (mbid, name) in playlist_artists {
            source_artists.push(name.clone());

            // Search Qobuz for tracks by this playlist artist (similarity = 1.0)
            // Fetch many more tracks since many might already be in playlist
            // For a playlist with 23 tracks, we need to search beyond those to find new ones
            let playlist_artist_limit = (self.config.tracks_per_artist * 5).max(30); // At least 30 tracks
            log::info!(
                "[SuggestionsEngine] Step 4a: Searching for '{}' (MBID: {}) with limit {}",
                name,
                mbid,
                playlist_artist_limit
            );
            let tracks = self
                .search_artist_tracks_with_limit(mbid, Some(name), 1.0, playlist_artist_limit)
                .await;
            log::info!(
                "[SuggestionsEngine] Step 4a: Found {} tracks for '{}'",
                tracks.len(),
                name
            );

            let mut added = 0;
            let mut skipped = 0;
            for mut track in tracks {
                // Skip if already in playlist
                if exclude_track_ids.contains(&track.track_id) {
                    skipped += 1;
                    continue;
                }

                if include_reasons {
                    track.reason = Some(format!("More from {}", name));
                }

                all_tracks.push(track);
                added += 1;
            }
            log::info!("[SuggestionsEngine] Step 4a: Added {} tracks for '{}' ({} skipped as already in playlist)", added, name, skipped);
        }
        log::info!(
            "[SuggestionsEngine] Step 4a completed in {:?}, got {} tracks from playlist artists",
            step4a_start.elapsed(),
            all_tracks.len()
        );
    }
}
