//! Login/logout: the credential-flow half of auth (see `auth.rs` for
//! client init/lazy-rebuild and session presence).

use qbz_models::{CoreEvent, FrontendAdapter, UserSession};

use crate::error::CoreError;

use super::QbzCore;

impl<A: FrontendAdapter + Send + Sync + 'static> QbzCore<A> {
    /// Login with email and password
    pub async fn login(&self, email: &str, password: &str) -> Result<UserSession, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;

        match client.login(email, password).await {
            Ok(session) => {
                self.emit(CoreEvent::LoggedIn {
                    session: session.clone(),
                })
                .await;
                Ok(session)
            }
            Err(e) => {
                self.emit(CoreEvent::Error {
                    code: "AUTH_FAILED".to_string(),
                    message: e.to_string(),
                    recoverable: true,
                })
                .await;
                Err(CoreError::AuthFailed(e.to_string()))
            }
        }
    }

    /// Restore a session from a saved OAuth user_auth_token.
    pub async fn login_with_token(&self, token: &str) -> Result<UserSession, CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;

        match client.login_with_token(token).await {
            Ok(session) => {
                self.emit(CoreEvent::LoggedIn {
                    session: session.clone(),
                })
                .await;
                Ok(session)
            }
            Err(e) => {
                self.emit(CoreEvent::Error {
                    code: "OAUTH_TOKEN_FAILED".to_string(),
                    message: e.to_string(),
                    recoverable: true,
                })
                .await;
                // Preserve the typed ApiError: callers must distinguish an
                // explicit auth rejection (clear the saved token) from a
                // network-class failure (keep it) — stringifying here made
                // that impossible and caused the token-clearing-on-boot bug.
                Err(CoreError::Api(e))
            }
        }
    }

    /// Inject an already-authenticated session (e.g. from OAuth flow).
    /// Emits a LoggedIn event so the rest of the system knows auth state changed.
    pub async fn set_session(&self, session: UserSession) -> Result<(), CoreError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(CoreError::NotInitialized)?;
        client.set_session(session.clone()).await;
        self.emit(CoreEvent::LoggedIn { session }).await;
        Ok(())
    }

    /// Logout the current user
    pub async fn logout(&self) -> Result<(), CoreError> {
        let client = self.client.read().await;
        if let Some(c) = client.as_ref() {
            c.logout().await;
            self.emit(CoreEvent::LoggedOut).await;
        }
        Ok(())
    }
}
