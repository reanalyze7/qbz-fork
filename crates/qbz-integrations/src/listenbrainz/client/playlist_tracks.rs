//! One curated playlist's tracks (JSPF).

use super::identifier::last_identifier_segment;
use super::{ListenBrainzClient, LISTENBRAINZ_API_URL};
use crate::error::{IntegrationError, IntegrationResult};
use crate::listenbrainz::models::*;

impl ListenBrainzClient {
    /// Fetch one playlist's tracks (JSPF).
    ///
    /// `GET /playlist/{playlist_mbid}`
    ///
    /// Public read; the `Authorization` header is sent only when a token is
    /// configured. HTTP 204/404 and empty/malformed bodies are treated as
    /// "no data" -> `Ok(vec![])`.
    ///
    /// Response shape: `{ playlist: { track: [ ... ] } }`. For each track:
    /// - `title`
    /// - `creator` -> `artist_name`
    /// - `album` -> `release_name`
    /// - `recording_mbid` = LAST path segment of the track `identifier`
    ///   (string OR array)
    /// - `caa_id` + `caa_release_mbid` from
    ///   `extension["https://musicbrainz.org/doc/jspf#track"].additional_metadata`
    pub async fn get_playlist_tracks(
        &self,
        playlist_mbid: &str,
    ) -> IntegrationResult<Vec<LbPlaylistTrack>> {
        let token = self.config.lock().await.token.clone();

        let url = format!("{}/playlist/{}", LISTENBRAINZ_API_URL, playlist_mbid);

        let mut request = self.client.get(&url);
        if let Some(token) = token {
            request = request.header("Authorization", format!("Token {}", token));
        }

        let response = request.send().await?;
        let status = response.status();

        if status == reqwest::StatusCode::NO_CONTENT || status == reqwest::StatusCode::NOT_FOUND {
            return Ok(vec![]);
        }
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(IntegrationError::internal(format!(
                "ListenBrainz playlist tracks failed: {} - {}",
                status, text
            )));
        }

        let body = response.text().await.unwrap_or_default();
        if body.trim().is_empty() {
            return Ok(vec![]);
        }
        let json: serde_json::Value = match serde_json::from_str(&body) {
            Ok(value) => value,
            Err(_) => return Ok(vec![]),
        };

        let tracks = json
            .get("playlist")
            .and_then(|playlist| playlist.get("track"))
            .and_then(|track| track.as_array())
            .cloned()
            .unwrap_or_default();

        let mut parsed = Vec::with_capacity(tracks.len());
        for track in tracks {
            let title = track
                .get("title")
                .and_then(|value| value.as_str())
                .unwrap_or_default()
                .to_string();
            let artist_name = track
                .get("creator")
                .and_then(|value| value.as_str())
                .unwrap_or_default()
                .to_string();
            let release_name = track
                .get("album")
                .and_then(|value| value.as_str())
                .map(|value| value.to_string());
            let recording_mbid = track.get("identifier").and_then(last_identifier_segment);

            let additional = track
                .get("extension")
                .and_then(|ext| ext.get("https://musicbrainz.org/doc/jspf#track"))
                .and_then(|ext| ext.get("additional_metadata"));
            let caa_id = additional
                .and_then(|meta| meta.get("caa_id"))
                .and_then(|value| value.as_i64());
            let caa_release_mbid = additional
                .and_then(|meta| meta.get("caa_release_mbid"))
                .and_then(|value| value.as_str())
                .map(|value| value.to_string());

            parsed.push(LbPlaylistTrack {
                recording_mbid,
                title,
                artist_name,
                release_name,
                caa_id,
                caa_release_mbid,
            });
        }

        Ok(parsed)
    }
}
