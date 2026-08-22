use super::MusicBrainzClient;
use crate::error::{IntegrationError, IntegrationResult};

impl MusicBrainzClient {
    pub(super) async fn check_enabled(&self) -> IntegrationResult<()> {
        if !self.is_enabled().await {
            return Err(IntegrationError::ServiceUnavailable(
                "MusicBrainz integration is disabled".into(),
            ));
        }
        Ok(())
    }

    #[allow(unused)]
    pub(super) async fn check_response(&self, _response: &reqwest::Response) {
        // Placeholder for response logging/metrics
    }

    pub(super) async fn handle_response_status(
        &self,
        response: reqwest::Response,
    ) -> IntegrationResult<reqwest::Response> {
        if response.status().is_success() {
            return Ok(response);
        }
        let status = response.status();
        // 429 (proxy-translated) and 503 (direct MusicBrainz) both signal that
        // the rate limit was hit. Surface the server's Retry-After so the caller
        // can back off instead of treating it as a generic error.
        if matches!(status.as_u16(), 429 | 503) {
            return Err(IntegrationError::RateLimited(Self::parse_retry_after(
                &response,
            )));
        }
        let text = response.text().await.unwrap_or_default();
        Err(IntegrationError::internal(format!(
            "MusicBrainz API error {}: {}",
            status, text
        )))
    }

    /// Parse the `Retry-After` header (whole seconds). MusicBrainz sends it on
    /// HTTP 503 when the per-IP rate limit is exceeded. Defaults to 1s because
    /// MB's per-IP limiter recovers within ~1 second.
    pub(super) fn parse_retry_after(response: &reqwest::Response) -> u64 {
        response
            .headers()
            .get(reqwest::header::RETRY_AFTER)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.trim().parse::<u64>().ok())
            .filter(|&s| s > 0)
            .unwrap_or(1)
    }

    /// Escape special characters in Lucene queries
    pub(super) fn escape_query(s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace(':', "\\:")
            .replace('(', "\\(")
            .replace(')', "\\)")
            .replace('[', "\\[")
            .replace(']', "\\]")
            .replace('{', "\\{")
            .replace('}', "\\}")
            .replace('^', "\\^")
            .replace('~', "\\~")
            .replace('*', "\\*")
            .replace('?', "\\?")
            .replace('!', "\\!")
            .replace('+', "\\+")
            .replace('-', "\\-")
            .replace('&', "\\&")
            .replace('|', "\\|")
    }
}
