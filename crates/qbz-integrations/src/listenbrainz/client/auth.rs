//! Config/session lifecycle: enable/disable, token validate/set/restore, disconnect.

use super::{ListenBrainzClient, LISTENBRAINZ_API_URL};
use crate::error::{IntegrationError, IntegrationResult};
use crate::listenbrainz::models::*;

impl ListenBrainzClient {
    /// Check if ListenBrainz integration is enabled
    pub async fn is_enabled(&self) -> bool {
        self.config.lock().await.enabled
    }

    /// Enable or disable ListenBrainz integration
    pub async fn set_enabled(&self, enabled: bool) {
        self.config.lock().await.enabled = enabled;
    }

    /// Check if authenticated (has valid token)
    pub async fn is_authenticated(&self) -> bool {
        let config = self.config.lock().await;
        config.token.is_some() && config.user_name.is_some()
    }

    /// Get current status
    pub async fn get_status(&self) -> ListenBrainzStatus {
        let config = self.config.lock().await;
        ListenBrainzStatus {
            connected: config.token.is_some() && config.user_name.is_some(),
            user_name: config.user_name.clone(),
            enabled: config.enabled,
        }
    }

    /// Set user token and validate it
    pub async fn set_token(&self, token: &str) -> IntegrationResult<UserInfo> {
        // Validate token first
        let validation = self.validate_token(token).await?;

        if !validation.valid {
            return Err(IntegrationError::AuthFailed(validation.message));
        }

        let user_name = validation.user_name.ok_or_else(|| {
            IntegrationError::AuthFailed("Token valid but no username returned".into())
        })?;

        // Store validated token and username
        {
            let mut config = self.config.lock().await;
            config.token = Some(token.to_string());
            config.user_name = Some(user_name.clone());
        }

        log::info!("ListenBrainz connected");

        Ok(UserInfo { user_name })
    }

    /// Restore token from saved session (without re-validating)
    pub async fn restore_token(&self, token: String, user_name: String) {
        let mut config = self.config.lock().await;
        config.token = Some(token);
        config.user_name = Some(user_name);
    }

    /// Get current token (for persistence)
    pub async fn get_token(&self) -> Option<String> {
        self.config.lock().await.token.clone()
    }

    /// Get current username
    pub async fn get_user_name(&self) -> Option<String> {
        self.config.lock().await.user_name.clone()
    }

    /// Disconnect (clear token)
    pub async fn disconnect(&self) {
        let mut config = self.config.lock().await;
        config.token = None;
        config.user_name = None;
        log::info!("ListenBrainz disconnected");
    }

    /// Validate a token with ListenBrainz API
    async fn validate_token(&self, token: &str) -> IntegrationResult<TokenValidationResponse> {
        let url = format!("{}/validate-token", LISTENBRAINZ_API_URL);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Token {}", token))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(IntegrationError::AuthFailed(format!(
                "Token validation failed: {} - {}",
                status, text
            )));
        }

        response
            .json::<TokenValidationResponse>()
            .await
            .map_err(Into::into)
    }
}
