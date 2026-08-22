use reqwest::StatusCode;

use super::QobuzClient;
use crate::endpoints;
use crate::error::{ApiError, Result};
use qbz_models::UserSession;

impl QobuzClient {
    /// Exchange an OAuth code for a full user session.
    ///
    /// This implements the new Qobuz OAuth flow:
    /// 1. GET /oauth/callback?code=CODE&private_key=KEY → { token }
    /// 2. POST /user/login with X-User-Auth-Token: token, body=extra=partner → UserSession
    pub async fn login_with_oauth_code(&self, code: &str) -> Result<UserSession> {
        use reqwest::header::{HeaderMap, HeaderValue};

        let tokens = self.tokens.read().await;
        let tokens = tokens
            .as_ref()
            .ok_or_else(|| ApiError::BundleExtractionError("Client not initialized".to_string()))?;
        let app_id = tokens.app_id.clone();
        let private_key = tokens.private_key.clone().ok_or_else(|| {
            ApiError::BundleExtractionError("OAuth private key not available in bundle".to_string())
        })?;
        let _ = tokens; // drop read lock

        // Step 1: Exchange code for token
        let callback_url = endpoints::build_url(endpoints::paths::OAUTH_CALLBACK);
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-App-Id",
            HeaderValue::from_str(&app_id).map_err(|_| ApiError::InvalidAppId)?,
        );

        log::info!("[OAuth] Exchanging code for token via /oauth/callback");
        // Auth exemption: raw client, bypasses the offline gate (sign-in is
        // explicit user intent to reach Qobuz; the gate governs services).
        let callback_response = self
            .http
            .get(&callback_url)
            .headers(headers)
            .query(&[
                ("code", code),
                ("private_key", &private_key),
                ("app_id", &app_id),
            ])
            .send()
            .await?;

        if !callback_response.status().is_success() {
            return Err(ApiError::ApiResponse(format!(
                "OAuth callback failed with status {}",
                callback_response.status()
            )));
        }

        let callback_json: serde_json::Value = callback_response.json().await?;
        let token = callback_json["token"]
            .as_str()
            .ok_or_else(|| {
                ApiError::ApiResponse("OAuth callback: no token in response".to_string())
            })?
            .to_string();

        log::info!("[OAuth] Got token, fetching user session via /user/login");

        // Step 2: Fetch user session using the token
        let user_login_url = endpoints::build_url(endpoints::paths::USER_LOGIN);
        let mut auth_headers = HeaderMap::new();
        auth_headers.insert(
            "X-App-Id",
            HeaderValue::from_str(&app_id).map_err(|_| ApiError::InvalidAppId)?,
        );
        auth_headers.insert(
            "X-User-Auth-Token",
            HeaderValue::from_str(&token)
                .map_err(|_| ApiError::AuthenticationError("Invalid OAuth token format".into()))?,
        );

        // Auth exemption: raw client (see /oauth/callback step above).
        let login_response = self
            .http
            .post(&user_login_url)
            .headers(auth_headers)
            .header("Content-Type", "text/plain;charset=UTF-8")
            .body("extra=partner")
            .send()
            .await?;

        match login_response.status() {
            StatusCode::OK => {
                let json: serde_json::Value = login_response.json().await?;
                let session = crate::auth::parse_login_response(&json)?;
                *self.session.write().await = Some(session.clone());
                log::info!("[OAuth] Session established");
                Ok(session)
            }
            StatusCode::UNAUTHORIZED => Err(ApiError::AuthenticationError(
                "OAuth token rejected by user/login".to_string(),
            )),
            status => Err(ApiError::ApiResponse(format!(
                "user/login OAuth step failed with status {}",
                status
            ))),
        }
    }

}
