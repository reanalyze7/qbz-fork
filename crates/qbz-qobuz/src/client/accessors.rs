use reqwest::Client;

use super::QobuzClient;
use crate::error::{ApiError, Result};

impl QobuzClient {
    /// Set the locale for API requests
    pub async fn set_locale(&self, locale: String) {
        *self.locale.write().await = locale;
    }

    /// Get the current locale (public for cache key generation)
    pub async fn get_locale(&self) -> String {
        self.locale.read().await.clone()
    }

    /// Get the current locale (internal use)
    pub(super) async fn locale(&self) -> String {
        self.locale.read().await.clone()
    }

    /// Get app ID (public for catalog search)
    pub async fn app_id(&self) -> Result<String> {
        self.tokens
            .read()
            .await
            .as_ref()
            .map(|t| t.app_id.clone())
            .ok_or_else(|| ApiError::BundleExtractionError("Client not initialized".to_string()))
    }

    /// Get HTTP client reference (public for catalog search)
    pub fn get_http(&self) -> &Client {
        &self.http
    }

    /// The single offline choke point (D3): every Qobuz SERVICE request flows
    /// through here. While offline mode is active, fail fast with a typed,
    /// non-transient error instead of timing out against the network.
    ///
    /// Exemption: the sign-in methods (`login`, `login_with_oauth_code`,
    /// `login_with_token`) use the raw `self.http` field instead —
    /// user-initiated authentication is an explicit intent to reach Qobuz;
    /// the offline gate governs services, not sign-in. Without the exemption
    /// a closed gate (induced offline, or a stale offline session) refuses
    /// the very login that would resolve it.
    pub(crate) fn http(&self) -> Result<&Client> {
        if crate::offline_gate::is_offline() {
            return Err(ApiError::OfflineMode);
        }
        Ok(&self.http)
    }
}
