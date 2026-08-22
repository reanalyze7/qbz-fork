use super::QobuzClient;
use crate::error::{ApiError, Result};
use qbz_models::*;

impl QobuzClient {
    /// Get stream URL with quality fallback
    pub async fn get_stream_url_with_fallback(
        &self,
        track_id: u64,
        preferred: Quality,
    ) -> Result<StreamUrl> {
        log::info!(
            "Getting stream URL with fallback for track {}, preferred quality: {:?}",
            track_id,
            preferred
        );
        let qualities = Quality::fallback_order();
        let start_idx = qualities.iter().position(|q| *q == preferred).unwrap_or(0);

        let mut track_unavailable = false;

        for quality in &qualities[start_idx..] {
            log::info!("Trying quality: {:?}", quality);
            match self.get_stream_url(track_id, *quality).await {
                Ok(url) if !url.has_restrictions() => {
                    log::info!(
                        "Got stream URL for requested quality format_id={}",
                        quality.id()
                    );
                    return Ok(url);
                }
                Ok(_) => {
                    log::info!("Quality {:?} has restrictions, trying next", quality);
                    continue;
                }
                Err(ApiError::InvalidAppSecret) => {
                    log::error!("Invalid app secret");
                    return Err(ApiError::InvalidAppSecret);
                }
                // A 403 (or an open breaker, or a 401) is NOT a per-quality
                // restriction — every quality would 403 the same way. Abort the
                // whole fallback loop immediately instead of firing 5 more
                // requests per track and feeding the storm (issue #637).
                Err(e @ (ApiError::Forbidden(_)
                | ApiError::ForbiddenCircuitOpen(_)
                | ApiError::AuthenticationError(_))) => {
                    log::warn!("Stream URL aborting quality fallback: {}", e);
                    return Err(e);
                }
                Err(ApiError::TrackUnavailable(_)) => {
                    // Track is completely unavailable on Qobuz
                    track_unavailable = true;
                    continue;
                }
                Err(e) => {
                    log::warn!("Quality {:?} failed: {}, trying next", quality, e);
                    continue;
                }
            }
        }

        // If all quality levels reported track unavailable, return that specific error
        if track_unavailable {
            log::error!("Track {} is no longer available on Qobuz", track_id);
            return Err(ApiError::TrackUnavailable(track_id));
        }

        log::error!("No quality available for track {}", track_id);
        Err(ApiError::NoQualityAvailable)
    }
}
