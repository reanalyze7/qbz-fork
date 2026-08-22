//! Collaborative-filtering recommendations endpoint.

use super::{ListenBrainzClient, LISTENBRAINZ_API_URL};
use crate::error::{IntegrationError, IntegrationResult};
use crate::listenbrainz::models::*;

impl ListenBrainzClient {
    /// Collaborative-filtering recommendations (PRIMARY personalized recommender).
    ///
    /// `GET /cf/recommendation/user/{user_name}/recording?count={count}`
    ///
    /// The `Authorization` header is sent only when a token is configured (raises
    /// rate limits but is not required — this is a public read keyed by username).
    /// HTTP 204/404 and empty bodies are treated as "no data" -> `Ok(vec![])`.
    /// Parses `payload.mbids[]` into `{recording_mbid, score, latest_listened_at}`.
    /// `latest_listened_at` is an ISO-8601 string OR null (null/absent = never listened).
    pub async fn get_cf_recommendations(
        &self,
        user_name: &str,
        count: u32,
    ) -> IntegrationResult<Vec<CfRecommendation>> {
        let token = self.config.lock().await.token.clone();

        let url = format!(
            "{}/cf/recommendation/user/{}/recording",
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

        // 204 No Content / 404 Not Found -> the user simply has no recommendations.
        if status == reqwest::StatusCode::NO_CONTENT || status == reqwest::StatusCode::NOT_FOUND {
            return Ok(vec![]);
        }
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(IntegrationError::internal(format!(
                "ListenBrainz CF recommendations failed: {} - {}",
                status, text
            )));
        }

        let body = response.text().await.unwrap_or_default();
        if body.trim().is_empty() {
            return Ok(vec![]);
        }
        // Defensive parse: tolerate any malformed payload as "no data".
        let json: serde_json::Value = match serde_json::from_str(&body) {
            Ok(value) => value,
            Err(_) => return Ok(vec![]),
        };

        let mbids = json
            .get("payload")
            .and_then(|payload| payload.get("mbids"))
            .and_then(|mbids| mbids.as_array())
            .cloned()
            .unwrap_or_default();

        let mut recommendations = Vec::with_capacity(mbids.len());
        for item in mbids {
            let recording_mbid = match item
                .get("recording_mbid")
                .and_then(|value| value.as_str())
            {
                Some(mbid) if !mbid.is_empty() => mbid.to_string(),
                // An entry with no recording_mbid is useless downstream; skip it.
                _ => continue,
            };
            let score = item.get("score").and_then(|value| value.as_f64()).unwrap_or(0.0);
            let latest_listened_at = item
                .get("latest_listened_at")
                .and_then(|value| value.as_str())
                .map(|value| value.to_string());

            recommendations.push(CfRecommendation {
                recording_mbid,
                score,
                latest_listened_at,
            });
        }

        Ok(recommendations)
    }
}
