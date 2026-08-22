use reqwest::StatusCode;
use serde_json::Value;

use super::QobuzClient;
use crate::auth::{parse_login_response, sign_get_file_url, get_timestamp};
use crate::endpoints::{self, paths};
use crate::error::{ApiError, Result};
use qbz_models::UserSession;

impl QobuzClient {
    /// Get validated secret (validates on first use)
    pub(crate) async fn secret(&self) -> Result<String> {
        // Check if we already have a validated secret
        if let Some(secret) = self.validated_secret.read().await.clone() {
            return Ok(secret);
        }

        // Need to validate secrets
        let tokens = self.tokens.read().await;
        let tokens = tokens
            .as_ref()
            .ok_or_else(|| ApiError::BundleExtractionError("Client not initialized".to_string()))?;

        for secret in &tokens.secrets {
            if self.test_secret(secret).await? {
                *self.validated_secret.write().await = Some(secret.clone());
                return Ok(secret.clone());
            }
        }

        Err(ApiError::InvalidAppSecret)
    }

    /// Test if a secret is valid using a known track
    async fn test_secret(&self, secret: &str) -> Result<bool> {
        let test_track_id = 5966783u64; // Known test track
        let timestamp = get_timestamp();
        let signature = sign_get_file_url(test_track_id, 5, timestamp, secret);

        let url = endpoints::build_url(paths::TRACK_GET_FILE_URL);
        let response = self
            .http()?
            .get(&url)
            .headers(self.api_headers().await?)
            .query(&[
                ("track_id", test_track_id.to_string()),
                ("format_id", "5".to_string()),
                ("intent", "stream".to_string()),
                ("request_ts", timestamp.to_string()),
                ("request_sig", signature),
            ])
            .send()
            .await?;

        Ok(response.status() != StatusCode::BAD_REQUEST)
    }

    /// Login with email and password
    pub async fn login(&self, email: &str, password: &str) -> Result<UserSession> {
        let url = endpoints::build_url(paths::USER_LOGIN);
        // Auth exemption: raw client, bypasses the offline gate (sign-in is
        // explicit user intent to reach Qobuz; the gate governs services).
        let response = self
            .http
            .get(&url)
            .headers(self.api_headers().await?)
            .query(&[("email", email), ("password", password)])
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let json: Value = response.json().await?;
                let session = parse_login_response(&json)?;
                *self.session.write().await = Some(session.clone());
                Ok(session)
            }
            StatusCode::UNAUTHORIZED => Err(ApiError::AuthenticationError(
                "Invalid credentials".to_string(),
            )),
            StatusCode::BAD_REQUEST => Err(ApiError::InvalidAppId),
            status => Err(ApiError::ApiResponse(format!(
                "Unexpected status: {}",
                status
            ))),
        }
    }

    /// Check if logged in
    pub async fn is_logged_in(&self) -> bool {
        self.session.read().await.is_some()
    }

    /// Logout - clear the session
    pub async fn logout(&self) {
        *self.session.write().await = None;
    }

    /// Inject an already-authenticated session (e.g. from OAuth flow).
    /// Use this when the session was obtained outside this client instance.
    pub async fn set_session(&self, session: UserSession) {
        *self.session.write().await = Some(session);
    }

    /// Get current user info (display name, subscription, and expiry if available)
    pub async fn get_user_info(&self) -> Option<(String, String, Option<String>)> {
        self.session.read().await.as_ref().map(|s| {
            (
                s.display_name.clone(),
                s.subscription_label.clone(),
                s.subscription_valid_until.clone(),
            )
        })
    }

    /// Get user auth token header value (public for catalog search)
    pub async fn auth_token(&self) -> Result<String> {
        self.session
            .read()
            .await
            .as_ref()
            .map(|s| s.user_auth_token.clone())
            .ok_or_else(|| ApiError::AuthenticationError("Not logged in".to_string()))
    }
}
