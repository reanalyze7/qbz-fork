//! Last.fm authentication flow: request token + session key exchange.

use serde_json::json;

use super::client::{LastFmClient, LASTFM_PROXY_URL};
use super::models::{AuthGetSessionResponse, AuthGetTokenResponse, LastFmResponse, LastFmSession};
use crate::error::{IntegrationError, IntegrationResult};

impl LastFmClient {
    /// Get a request token and authorization URL for authentication
    ///
    /// Returns: (token, auth_url)
    ///
    /// The user should be directed to auth_url to authorize the application.
    /// Once authorized, call `get_session` with the token to complete authentication.
    pub async fn get_token(&self) -> IntegrationResult<(String, String)> {
        let url = format!("{}/auth.getToken", LASTFM_PROXY_URL);

        let response = self.client.post(&url).json(&json!({})).send().await?;

        let data: LastFmResponse<AuthGetTokenResponse> = response.json().await?;

        match data {
            LastFmResponse::Success(r) => {
                let auth_url = r
                    .auth_url
                    .unwrap_or_else(|| format!("https://www.last.fm/api/auth/?token={}", r.token));
                Ok((r.token, auth_url))
            }
            LastFmResponse::Error { error, message } => Err(IntegrationError::api(error, message)),
        }
    }

    /// Get session key after user has authorized
    ///
    /// Call this after the user has visited the auth_url from `get_token`.
    pub async fn get_session(&mut self, token: &str) -> IntegrationResult<LastFmSession> {
        // Never log request-token material (even a prefix) — support log
        // bundles must not carry auth substrings.
        log::info!("Requesting Last.fm session");

        let url = format!("{}/auth.getSession", LASTFM_PROXY_URL);

        let response = self
            .client
            .post(&url)
            .json(&json!({ "token": token }))
            .send()
            .await?;

        let response_text = response.text().await?;
        // Do not log the raw body: it includes the session key.
        let data: LastFmResponse<AuthGetSessionResponse> = serde_json::from_str(&response_text)?;

        match data {
            LastFmResponse::Success(r) => {
                log::info!("Last.fm session obtained for user: {}", r.session.name);
                self.session_key = Some(r.session.key.clone());
                Ok(r.session)
            }
            LastFmResponse::Error { error, message } => {
                log::error!("Last.fm auth error {}: {}", error, message);
                Err(IntegrationError::api(error, message))
            }
        }
    }
}
