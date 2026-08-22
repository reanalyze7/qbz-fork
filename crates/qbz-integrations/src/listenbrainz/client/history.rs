//! Raw scrobble history endpoint.

use super::{ListenBrainzClient, LISTENBRAINZ_API_URL};
use crate::error::{IntegrationError, IntegrationResult};
use crate::listenbrainz::models::*;

impl ListenBrainzClient {
    /// Raw scrobble history with timestamps.
    ///
    /// `GET /user/{user_name}/listens?count={count}`
    ///
    /// The `Authorization` header is sent only when a token is configured.
    /// HTTP 204/404 and empty bodies are treated as "no data" -> `Ok(vec![])`.
    /// Parses `payload.listens[]` into
    /// `{listened_at, track_metadata.artist_name, track_metadata.track_name,
    ///   track_metadata.mbid_mapping.recording_mbid}`.
    pub async fn get_recent_listens(
        &self,
        user_name: &str,
        count: u32,
    ) -> IntegrationResult<Vec<LbListen>> {
        let token = self.config.lock().await.token.clone();

        let url = format!("{}/user/{}/listens", LISTENBRAINZ_API_URL, user_name);

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
                "ListenBrainz recent listens failed: {} - {}",
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

        let listens = json
            .get("payload")
            .and_then(|payload| payload.get("listens"))
            .and_then(|listens| listens.as_array())
            .cloned()
            .unwrap_or_default();

        let mut parsed = Vec::with_capacity(listens.len());
        for item in listens {
            let listened_at = item
                .get("listened_at")
                .and_then(|value| value.as_i64())
                .unwrap_or(0);
            let track_metadata = item.get("track_metadata");
            let artist_name = track_metadata
                .and_then(|meta| meta.get("artist_name"))
                .and_then(|value| value.as_str())
                .unwrap_or_default()
                .to_string();
            let track_name = track_metadata
                .and_then(|meta| meta.get("track_name"))
                .and_then(|value| value.as_str())
                .unwrap_or_default()
                .to_string();
            let recording_mbid = track_metadata
                .and_then(|meta| meta.get("mbid_mapping"))
                .and_then(|mapping| mapping.get("recording_mbid"))
                .and_then(|value| value.as_str())
                .map(|value| value.to_string());

            parsed.push(LbListen {
                listened_at,
                artist_name,
                track_name,
                recording_mbid,
            });
        }

        Ok(parsed)
    }
}
