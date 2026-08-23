// ============================ IO helpers ============================
mod http;
mod listener;

pub(super) use http::http_request_2xx;
pub(super) use listener::{bind_login_listener, capture_callback};

use qbz_app::shell::AppRuntime;
use qbz_audio::settings::AudioSettings;
use qbz_core::NoOpAdapter;
use qbz_models::UserSession;

use super::error::LoginError;
use crate::paths::ProfileRoots;

/// Compose the minimal client-only runtime: a headless [`NoOpAdapter`] and
/// default audio settings (no store is opened — login never touches audio),
/// then `init()` to extract the Qobuz bundle tokens the sign-in calls need.
pub(super) async fn build_login_runtime() -> Result<AppRuntime<NoOpAdapter>, LoginError> {
    let runtime =
        AppRuntime::with_audio_settings(NoOpAdapter, None, AudioSettings::default(), None);
    if let Err(e) = runtime.init().await {
        return Err(LoginError::Failed(format!(
            "could not reach Qobuz to start login: {e}\n  → check your connection and retry"
        )));
    }
    Ok(runtime)
}

pub(super) async fn read_app_id(runtime: &AppRuntime<NoOpAdapter>) -> Result<String, LoginError> {
    let client_lock = runtime.core().client();
    let guard = client_lock.read().await;
    let client = guard.as_ref().ok_or_else(|| {
        LoginError::Failed(
            "Qobuz client not initialized — could not reach Qobuz\n  \
             → check your connection and retry"
                .to_string(),
        )
    })?;
    client
        .app_id()
        .await
        .map_err(|e| LoginError::Failed(format!("could not read the Qobuz app id: {e}")))
}

pub(super) async fn exchange_code(
    runtime: &AppRuntime<NoOpAdapter>,
    code: &str,
) -> Result<UserSession, LoginError> {
    let client_lock = runtime.core().client();
    let guard = client_lock.read().await;
    let client = guard
        .as_ref()
        .ok_or_else(|| LoginError::Failed("Qobuz client not initialized".to_string()))?;
    client.login_with_oauth_code(code).await.map_err(super::error::map_api_err)
}

/// Register the secret, persist the token into the daemon config root (0600),
/// then best-effort nudge a running daemon. Persist happens ONLY here — after
/// the caller already live-validated the session.
pub(super) fn finalize(roots: &ProfileRoots, session: &UserSession) -> Result<(), LoginError> {
    // §6.3: register before the token can reach any log line (idempotent — the
    // token path already registered it in `validate_token`).
    qbz_log::register_secret(session.user_auth_token.clone());
    qbz_credentials::save_oauth_token_at(&roots.config, &session.user_auth_token).map_err(|e| {
        LoginError::Failed(format!(
            "could not save credentials to {}: {e}",
            roots.config.display()
        ))
    })?;
    let host = nudge_host(roots);
    // token: opt-in [server] token, wired by T6.
    let _ = super::entry::nudge_reload(&host, None);
    Ok(())
}

/// The local daemon's reload address. Credentials are written to the LOCAL
/// config root, so the daemon to nudge is always local; its port comes from the
/// same `qbzd.toml` the daemon reads (default 8182).
pub(crate) fn nudge_host(roots: &ProfileRoots) -> String {
    let port = crate::config::QbzdConfig::load(&roots.config.join("qbzd.toml"))
        .map(|(c, _)| c.server.port)
        .unwrap_or(8182);
    format!("127.0.0.1:{port}")
}

pub(super) fn read_stdin_line() -> Result<String, LoginError> {
    use std::io::BufRead;
    let mut line = String::new();
    std::io::stdin()
        .lock()
        .read_line(&mut line)
        .map_err(|e| LoginError::Failed(format!("could not read from stdin: {e}")))?;
    Ok(line)
}
