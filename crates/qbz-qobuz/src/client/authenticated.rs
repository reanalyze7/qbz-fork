use reqwest::StatusCode;
use serde_json::Value;

use super::{body_preview, QobuzClient};
use crate::auth::{get_timestamp, sign_get_file_url};
use crate::endpoints::{self, paths};
use crate::error::{ApiError, Result};
use qbz_models::*;

impl QobuzClient {
    // === Authenticated endpoints ===

    /// Get stream URL for a track (requires auth + signature)
    pub async fn get_stream_url(&self, track_id: u64, quality: Quality) -> Result<StreamUrl> {
        // Back off before the network if the 403 breaker is open (issue #637).
        self.forbidden_guard()?;
        log::info!(
            "Getting stream URL for track {} with quality {:?}",
            track_id,
            quality
        );
        let url = endpoints::build_url(paths::TRACK_GET_FILE_URL);
        let timestamp = get_timestamp();
        log::debug!("Getting secret for signing...");
        let secret = self.secret().await?;
        log::debug!("Secret obtained, signing request...");
        let signature = sign_get_file_url(track_id, quality.id(), timestamp, &secret);

        log::debug!("Sending stream URL request...");
        let response = self
            .http()?
            .get(&url)
            .headers(self.authenticated_headers().await?)
            .query(&[
                ("track_id", track_id.to_string()),
                ("format_id", quality.id().to_string()),
                ("intent", "stream".to_string()),
                ("request_ts", timestamp.to_string()),
                ("request_sig", signature),
            ])
            .send()
            .await?;

        let status = response.status();
        log::info!("Stream URL response status: {}", status);
        // Feed the breaker: a 403 counts toward opening it; a 200 resets it.
        self.note_forbidden_status(status);
        match status {
            StatusCode::OK => {
                let json: Value = response.json().await?;
                log::debug!(
                    "Stream URL response JSON keys: {:?}",
                    json.as_object().map(|o| o.keys().collect::<Vec<_>>())
                );

                // Check for restrictions
                let restrictions: Vec<StreamRestriction> = json
                    .get("restrictions")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_default();

                // Validate that we got an actual URL (track may be unavailable)
                let url = json["url"].as_str().unwrap_or("").to_string();
                if url.is_empty() {
                    // Log the restriction codes for debugging
                    let restriction_codes: Vec<&str> =
                        restrictions.iter().map(|r| r.code.as_str()).collect();
                    log::warn!(
                        "Stream URL missing for track {} - restrictions: {:?}",
                        track_id,
                        restriction_codes
                    );
                    return Err(ApiError::TrackUnavailable(track_id));
                }

                Ok(StreamUrl {
                    url,
                    format_id: json["format_id"].as_u64().unwrap_or(0) as u32,
                    mime_type: json["mime_type"].as_str().unwrap_or("").to_string(),
                    sampling_rate: json["sampling_rate"].as_f64().unwrap_or(0.0),
                    bit_depth: json["bit_depth"].as_u64().map(|v| v as u32),
                    track_id,
                    restrictions,
                })
            }
            StatusCode::BAD_REQUEST => Err(ApiError::InvalidAppSecret),
            StatusCode::UNAUTHORIZED => Err(ApiError::AuthenticationError(
                "stream URL 401 — user auth token invalid or expired".to_string(),
            )),
            StatusCode::FORBIDDEN => {
                let preview = body_preview(response).await;
                log::warn!("Stream URL 403 for track {}{}", track_id, preview);
                Err(ApiError::Forbidden(preview))
            }
            status => Err(ApiError::ApiResponse(format!(
                "Unexpected status: {}",
                status
            ))),
        }
    }
}
