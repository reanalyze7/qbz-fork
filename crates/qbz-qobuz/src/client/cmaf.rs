use super::{body_preview, QobuzClient};
use crate::auth::{get_timestamp, sign_file_url};
use crate::endpoints::{self, paths};
use crate::error::{ApiError, Result};
use qbz_models::*;

impl QobuzClient {
    // === CMAF streaming endpoints ===

    /// Get CMAF segmented file URL for a track.
    ///
    /// This is the new streaming endpoint that returns encrypted CMAF segments
    /// instead of a direct file URL.
    pub async fn get_file_url(
        &self,
        track_id: u64,
        quality: Quality,
    ) -> Result<TrackFileUrl> {
        let format_id = quality.id();
        let url = endpoints::build_url(paths::FILE_URL);

        // Back off before the network if the 403 breaker is open (issue #637).
        self.forbidden_guard()?;

        // Retry transient failures (5xx / 429 / network blips) with backoff so
        // a momentary hiccup on the next track's file/url is not turned into a
        // queue skip. A real 404 → TrackUnavailable is terminal and returns
        // immediately to the (bounded) skip path. Issue #467.
        crate::retry::retry_transient(
            crate::retry::DEFAULT_MAX_ATTEMPTS,
            "CMAF file/url",
            ApiError::is_transient,
            |_attempt| {
                let url = url.clone();
                async move {
                    let (session_id, _infos) = self.ensure_cmaf_session().await?;

                    // Fresh timestamp + signature per attempt — the request
                    // signature is time-bound and would expire across retries.
                    let timestamp = get_timestamp();
                    let sig = sign_file_url(track_id, format_id, timestamp);

                    let mut headers = self.authenticated_headers().await?;
                    headers.insert(
                        "X-Session-Id",
                        reqwest::header::HeaderValue::from_str(&session_id).map_err(|_| {
                            ApiError::ApiResponse("Invalid session ID format".into())
                        })?,
                    );

                    let response = self
                        .http()?
                        .get(&url)
                        .headers(headers)
                        .query(&[
                            ("track_id", track_id.to_string()),
                            ("format_id", format_id.to_string()),
                            ("intent", "stream".to_string()),
                            ("request_ts", timestamp.to_string()),
                            ("request_sig", sig),
                        ])
                        .send()
                        .await?;

                    let status = response.status();
                    log::info!(
                        "[CMAF] file/url track_id={} format_id={} status={}",
                        track_id,
                        format_id,
                        status
                    );
                    // Feed the breaker: 403 counts toward opening it; 2xx resets.
                    self.note_forbidden_status(status);

                    if !status.is_success() {
                        let code = status.as_u16();
                        if status == reqwest::StatusCode::FORBIDDEN {
                            let preview = body_preview(response).await;
                            log::warn!(
                                "[CMAF] file/url 403 for track {}{}",
                                track_id,
                                preview
                            );
                            return Err(ApiError::Forbidden(preview));
                        }
                        return Err(if code == 404 {
                            ApiError::TrackUnavailable(track_id)
                        } else if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                            // Honor Retry-After (seconds) when the server sends it.
                            let retry_after = response
                                .headers()
                                .get(reqwest::header::RETRY_AFTER)
                                .and_then(|v| v.to_str().ok())
                                .and_then(|s| s.trim().parse::<u64>().ok())
                                .unwrap_or(2);
                            ApiError::RateLimited(retry_after)
                        } else if status.is_server_error() {
                            ApiError::ServerError(code)
                        } else {
                            ApiError::ApiResponse(format!(
                                "file/url failed with status {}",
                                status
                            ))
                        });
                    }

                    let file_url: TrackFileUrl = response.json().await?;
                    log::info!(
                        "[CMAF] file/url result: segments={}, mime={:?}, sampling_rate={:?}",
                        file_url.n_segments,
                        file_url.mime_type,
                        file_url.sampling_rate
                    );

                    Ok(file_url)
                }
            },
        )
        .await
    }
}
