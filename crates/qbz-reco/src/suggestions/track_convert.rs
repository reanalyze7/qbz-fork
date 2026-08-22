//! Mapping Qobuz `Track` results into `SuggestedTrack`.
//!
//! `track_to_suggested` and `track_to_suggested_with_qobuz_id` are ~90%
//! duplicated (ported verbatim from Tauri); not merged here to avoid a
//! behavior change during a mechanical split — flagged for a follow-up.

use super::{SuggestedTrack, SuggestionsEngine};
use qbz_models::Track;

impl SuggestionsEngine {
    /// Convert a Track to a SuggestedTrack
    pub(super) fn track_to_suggested(
        &self,
        track: &Track,
        artist_mbid: &str,
        similarity: f32,
    ) -> SuggestedTrack {
        // Extract album info including image URL
        let (album_title, album_id, album_image_url) = match &track.album {
            Some(album) => {
                let image_url = album
                    .image
                    .thumbnail
                    .as_ref()
                    .or(album.image.small.as_ref())
                    .or(album.image.large.as_ref())
                    .cloned();
                (album.title.clone(), album.id.clone(), image_url)
            }
            None => (String::new(), String::new(), None),
        };

        // Extract artist name and ID from track performer
        let (track_artist, artist_id) = match &track.performer {
            Some(p) => (p.name.clone(), Some(p.id)),
            None => (String::new(), None),
        };

        SuggestedTrack {
            track_id: track.id,
            title: track.title.clone(),
            artist_name: track_artist,
            artist_id,
            artist_mbid: Some(artist_mbid.to_string()),
            album_title,
            album_id,
            album_image_url,
            duration: track.duration,
            similarity_score: similarity,
            reason: None,
        }
    }

    /// Convert a Track to a SuggestedTrack (using Qobuz artist ID instead of MBID)
    pub(super) fn track_to_suggested_with_qobuz_id(
        &self,
        track: &Track,
        _qobuz_artist_id: u64,
        similarity: f32,
    ) -> SuggestedTrack {
        let (album_title, album_id, album_image_url) = match &track.album {
            Some(album) => {
                let image_url = album
                    .image
                    .thumbnail
                    .as_ref()
                    .or(album.image.small.as_ref())
                    .or(album.image.large.as_ref())
                    .cloned();
                (album.title.clone(), album.id.clone(), image_url)
            }
            None => (String::new(), String::new(), None),
        };

        let (track_artist, artist_id) = match &track.performer {
            Some(p) => (p.name.clone(), Some(p.id)),
            None => (String::new(), None),
        };

        SuggestedTrack {
            track_id: track.id,
            title: track.title.clone(),
            artist_name: track_artist,
            artist_id,
            artist_mbid: None, // No MBID for Qobuz-sourced similar artists
            album_title,
            album_id,
            album_image_url,
            duration: track.duration,
            similarity_score: similarity,
            reason: None,
        }
    }
}
