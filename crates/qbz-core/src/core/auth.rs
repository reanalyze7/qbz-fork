//! Auth lifecycle: client init/lazy-rebuild and session presence.
//! Login/logout live in `login.rs`.

use qbz_models::FrontendAdapter;
use qbz_qobuz::QobuzClient;

use crate::error::CoreError;

use super::QbzCore;

impl<A: FrontendAdapter + Send + Sync + 'static> QbzCore<A> {
    /// Initialize the core
    ///
    /// This should be called once at startup to set up all subsystems.
    /// Best-effort: if bundle token extraction fails (e.g. no network), the
    /// core still finishes initialization so queue manager and player remain
    /// usable for offline/local playback. API calls will then return
    /// `CoreError::NotInitialized` until the client is rebuilt with tokens.
    pub async fn init(&self) -> Result<(), CoreError> {
        let mut initialized = self.initialized.write().await;
        if *initialized {
            return Ok(());
        }

        let client = QobuzClient::new().map_err(|e| CoreError::Internal(e.to_string()))?;

        match client.init().await {
            Ok(_) => {
                *self.client.write().await = Some(client);
                log::info!("QbzCore initialized with bundle tokens");
            }
            Err(e) => {
                log::warn!(
                    "QbzCore: bundle token extraction failed ({}). Starting in offline-tolerant mode; API calls will be unavailable until next online start.",
                    e
                );
            }
        }

        *initialized = true;
        Ok(())
    }

    /// Whether the Qobuz API client is initialized (bundle tokens extracted).
    /// Returns false when the core is running in offline-tolerant mode after
    /// a failed bundle extraction at startup.
    pub async fn is_api_initialized(&self) -> bool {
        self.client.read().await.is_some()
    }

    /// Best-effort attempt to rebuild the Qobuz API client. Useful when the
    /// initial `init()` ran offline and the host has since regained network.
    /// No-op when the client is already initialized.
    pub async fn try_init_api(&self) -> Result<(), CoreError> {
        {
            let guard = self.client.read().await;
            if guard.is_some() {
                return Ok(());
            }
        }

        let client = QobuzClient::new().map_err(|e| CoreError::Internal(e.to_string()))?;
        client
            .init()
            .await
            .map_err(|e| CoreError::Internal(format!("Failed to extract bundle tokens: {}", e)))?;
        *self.client.write().await = Some(client);
        log::info!("QbzCore: API client initialized lazily");
        Ok(())
    }

    /// Check if a user session exists
    pub async fn has_session(&self) -> bool {
        let client = self.client.read().await;
        if let Some(c) = client.as_ref() {
            c.is_logged_in().await
        } else {
            false
        }
    }
}
