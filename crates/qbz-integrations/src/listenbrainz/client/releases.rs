//! Personalized fresh releases endpoint.

use super::{ListenBrainzClient, LISTENBRAINZ_API_URL};
use crate::error::{IntegrationError, IntegrationResult};
use crate::listenbrainz::models::*;

impl ListenBrainzClient {
    /// Personalized fresh releases.
    ///
    /// `GET /user/{user_name}/fresh_releases?days={days}`
    ///
    /// `days` is clamped to `1..=90`. Public read; the `Authorization` header
    /// is sent only when a token is configured. HTTP 204/404 and empty/malformed
    /// bodies are treated as "no data" -> `Ok(vec![])`.
    ///
    /// Response shape: `{ payload: { releases: [ ... ] } }`. For each release:
    /// - `release_name`
    /// - `artist_credit_name`
    /// - `release_mbid`
    /// - `release_group_mbid`
    /// - `release_group_primary_type` -> `primary_type`
    /// - `caa_id`
    /// - `caa_release_mbid`
    /// - `release_date`
    /// - `listen_count`
    pub async fn get_fresh_releases(
        &self,
        user_name: &str,
        days: u32,
    ) -> IntegrationResult<Vec<LbFreshRelease>> {
        let token = self.config.lock().await.token.clone();

        let days = days.clamp(1, 90);

        let url = format!("{}/user/{}/fresh_releases", LISTENBRAINZ_API_URL, user_name);

        let mut request = self.client.get(&url).query(&[("days", days.to_string())]);
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
                "ListenBrainz fresh releases failed: {} - {}",
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

        let releases = json
            .get("payload")
            .and_then(|payload| payload.get("releases"))
            .and_then(|releases| releases.as_array())
            .cloned()
            .unwrap_or_default();

        let mut parsed = Vec::with_capacity(releases.len());
        for release in releases {
            let release_name = release
                .get("release_name")
                .and_then(|value| value.as_str())
                .unwrap_or_default()
                .to_string();
            let artist_credit_name = release
                .get("artist_credit_name")
                .and_then(|value| value.as_str())
                .unwrap_or_default()
                .to_string();
            let release_mbid = release
                .get("release_mbid")
                .and_then(|value| value.as_str())
                .map(|value| value.to_string());
            let release_group_mbid = release
                .get("release_group_mbid")
                .and_then(|value| value.as_str())
                .map(|value| value.to_string());
            let primary_type = release
                .get("release_group_primary_type")
                .and_then(|value| value.as_str())
                .map(|value| value.to_string());
            let caa_id = release.get("caa_id").and_then(|value| value.as_i64());
            let caa_release_mbid = release
                .get("caa_release_mbid")
                .and_then(|value| value.as_str())
                .map(|value| value.to_string());
            let release_date = release
                .get("release_date")
                .and_then(|value| value.as_str())
                .map(|value| value.to_string());
            let listen_count = release.get("listen_count").and_then(|value| value.as_u64());

            parsed.push(LbFreshRelease {
                release_name,
                artist_credit_name,
                release_mbid,
                release_group_mbid,
                primary_type,
                caa_id,
                caa_release_mbid,
                release_date,
                listen_count,
            });
        }

        Ok(parsed)
    }
}
