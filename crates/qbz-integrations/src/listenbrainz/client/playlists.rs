//! "Created for you" curated playlist listing.

use super::identifier::last_identifier_segment;
use super::{ListenBrainzClient, LISTENBRAINZ_API_URL};
use crate::error::{IntegrationError, IntegrationResult};
use crate::listenbrainz::models::*;

impl ListenBrainzClient {
    /// List the "Created for you" curated playlists (Weekly Jams = familiar,
    /// Weekly Exploration = discovery, Top Discoveries, etc.).
    ///
    /// `GET /user/{user_name}/playlists/createdfor?count={count}`
    ///
    /// Public read; the `Authorization` header is sent only when a token is
    /// configured. HTTP 204/404 and empty/malformed bodies are treated as
    /// "no data" -> `Ok(vec![])`.
    ///
    /// Response shape: `{ playlists: [ { playlist: {...} }, ... ] }`. For each
    /// inner `playlist` object:
    /// - `title`
    /// - `date` -> `created_at`
    /// - `annotation`
    /// - `playlist_mbid` = LAST path segment of `identifier` (string OR array;
    ///   e.g. `"https://listenbrainz.org/playlist/{mbid}"`)
    /// - `source_patch` = `extension["https://musicbrainz.org/doc/jspf#playlist"]`
    ///   `.additional_metadata.algorithm_metadata.source_patch`
    pub async fn get_created_for_playlists(
        &self,
        user_name: &str,
        count: u32,
    ) -> IntegrationResult<Vec<LbPlaylistMeta>> {
        let token = self.config.lock().await.token.clone();

        let url = format!(
            "{}/user/{}/playlists/createdfor",
            LISTENBRAINZ_API_URL, user_name
        );

        let mut request = self
            .client
            .get(&url)
            .query(&[("count", count.to_string())]);
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
                "ListenBrainz created-for playlists failed: {} - {}",
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

        let playlists = json
            .get("playlists")
            .and_then(|playlists| playlists.as_array())
            .cloned()
            .unwrap_or_default();

        let mut parsed = Vec::with_capacity(playlists.len());
        for wrapper in playlists {
            // Each array entry wraps the real object under a `playlist` key.
            let playlist = match wrapper.get("playlist") {
                Some(playlist) => playlist,
                None => continue,
            };

            let playlist_mbid = match playlist
                .get("identifier")
                .and_then(last_identifier_segment)
            {
                Some(mbid) => mbid,
                // No usable playlist id -> useless downstream; skip it.
                None => continue,
            };

            let title = playlist
                .get("title")
                .and_then(|value| value.as_str())
                .unwrap_or_default()
                .to_string();
            let created_at = playlist
                .get("date")
                .and_then(|value| value.as_str())
                .map(|value| value.to_string());
            let annotation = playlist
                .get("annotation")
                .and_then(|value| value.as_str())
                .map(|value| value.to_string());
            let source_patch = playlist
                .get("extension")
                .and_then(|ext| ext.get("https://musicbrainz.org/doc/jspf#playlist"))
                .and_then(|ext| ext.get("additional_metadata"))
                .and_then(|meta| meta.get("algorithm_metadata"))
                .and_then(|algo| algo.get("source_patch"))
                .and_then(|value| value.as_str())
                .map(|value| value.to_string());

            parsed.push(LbPlaylistMeta {
                playlist_mbid,
                title,
                source_patch,
                annotation,
                created_at,
            });
        }

        Ok(parsed)
    }
}
