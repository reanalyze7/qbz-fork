use reqwest::StatusCode;

use super::QobuzClient;
use crate::endpoints;
use crate::error::{ApiError, Result};
use qbz_models::UserSession;

impl QobuzClient {
    /// Restore a session from a previously saved OAuth user_auth_token.
    ///
    /// Used at startup when the user logged in via OAuth (web browser) and the
    /// token was persisted. Calls POST /user/login with X-User-Auth-Token header.
    /// Returns an error if the token has expired.
    pub async fn login_with_token(&self, token: &str) -> Result<UserSession> {
        use reqwest::header::{HeaderMap, HeaderValue};

        let tokens = self.tokens.read().await;
        let app_id = tokens
            .as_ref()
            .ok_or_else(|| ApiError::BundleExtractionError("Client not initialized".to_string()))?
            .app_id
            .clone();
        drop(tokens);

        let user_login_url = endpoints::build_url(endpoints::paths::USER_LOGIN);
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-App-Id",
            HeaderValue::from_str(&app_id).map_err(|_| ApiError::InvalidAppId)?,
        );
        headers.insert(
            "X-User-Auth-Token",
            HeaderValue::from_str(token)
                .map_err(|_| ApiError::AuthenticationError("Invalid token format".into()))?,
        );

        log::info!("[OAuth] Restoring session from saved token");
        // Auth exemption: raw client, bypasses the offline gate (sign-in is
        // explicit user intent to reach Qobuz; the gate governs services).
        let resp = self
            .http
            .post(&user_login_url)
            .headers(headers)
            .header("Content-Type", "text/plain;charset=UTF-8")
            .body("extra=partner")
            .send()
            .await?;

        match resp.status() {
            StatusCode::OK => {
                let json: serde_json::Value = resp.json().await?;
                let session = crate::auth::parse_login_response(&json)?;
                *self.session.write().await = Some(session.clone());
                log::info!("[OAuth] Session restored from token");
                Ok(session)
            }
            StatusCode::UNAUTHORIZED => Err(ApiError::AuthenticationError(
                "OAuth token expired or invalid".to_string(),
            )),
            status => Err(ApiError::ApiResponse(format!(
                "Token re-auth failed: {}",
                status
            ))),
        }
    }
}
