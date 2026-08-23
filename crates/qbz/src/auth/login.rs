//! `login_via_system_browser`: the full browser-based OAuth login.

use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;
use tokio::net::TcpListener;

use super::init::ensure_api_initialized;
use super::oauth_listener::capture_oauth_code;
use super::per_user::activate_per_user_stores;
use super::types::{LoginPhase, SessionInfo, OAUTH_TIMEOUT};

/// Run the full system-browser OAuth login. Returns the authenticated
/// session info on success. `on_phase` fires at each milestone (see
/// [`LoginPhase`]); it may be called from this async context, so it must
/// hop to the UI thread itself.
pub async fn login_via_system_browser<A, F>(
    runtime: &Arc<AppRuntime<A>>,
    on_phase: F,
) -> Result<SessionInfo, String>
where
    A: FrontendAdapter + Send + Sync + 'static,
    F: Fn(LoginPhase) + Send + Sync,
{
    let core = runtime.core();

    // NOTE: no offline_session pre-clear here. The sign-in endpoints are
    // EXEMPT from the offline gate (qbz-qobuz raw-client auth methods since
    // c207f232), so a live offline session never blocks the login itself —
    // and session activation (`runtime.activate`) is purely local. The old
    // upfront clear ended the offline session the moment the attempt
    // STARTED, which unlocked the shell (empty Discover/Library) while the
    // browser OAuth was still pending. The flag now drops on SUCCESS only
    // (below); the one still-gated dependency — a cold bundle-token init —
    // is scoped inside ensure_api_initialized.
    ensure_api_initialized(core).await?;

    let app_id = {
        let client_lock = core.client();
        let guard = client_lock.read().await;
        let client = guard.as_ref().ok_or("Qobuz client not initialized")?;
        client.app_id().await.map_err(|e| e.to_string())?
    };

    // One-shot local listener for the OAuth redirect.
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| format!("Failed to bind OAuth listener: {e}"))?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();

    let oauth_url = format!(
        "https://www.qobuz.com/signin/oauth?ext_app_id={}&redirect_url={}",
        app_id,
        urlencoding::encode(&format!("http://localhost:{port}")),
    );

    log::info!("[qbz-slint] opening system browser for OAuth (port {port})");
    open::that(&oauth_url).map_err(|e| format!("Failed to open browser: {e}"))?;
    on_phase(LoginPhase::WaitingForBrowser);

    let code = tokio::time::timeout(OAUTH_TIMEOUT, capture_oauth_code(listener))
        .await
        .map_err(|_| "OAuth login timed out".to_string())?
        .ok_or_else(|| "OAuth login cancelled or no code received".to_string())?;

    log::info!("[qbz-slint] OAuth code captured, exchanging for session");
    on_phase(LoginPhase::Authenticating);

    let session = {
        let client_lock = core.client();
        let guard = client_lock.read().await;
        let client = guard.as_ref().ok_or("Qobuz client not initialized")?;
        match client.login_with_oauth_code(&code).await {
            Ok(session) => session,
            Err(e) => {
                // D4 producer: ONLY an explicit ineligible-account verdict
                // starts the grace clock. Generic 401/network errors never do.
                if matches!(e, qbz_qobuz::ApiError::IneligibleUser) {
                    crate::offline_mode::subscription_mark_invalid();
                }
                return Err(e.to_string());
            }
        }
    };
    let user_id = session.user_id;
    let display_name = session.display_name.clone();
    let subscription = session.subscription_label.clone();
    let token = session.user_auth_token.clone();
    // Redaction (defense in depth): register the live auth token so any log line
    // that embeds it bare (e.g. a signed stream URL, a header dump) is scrubbed
    // before it reaches the in-memory ring, the on-disk log, or the clipboard.
    qbz_log::register_secret(token.clone());

    // Emit LoggedIn through the core (idempotent set_session).
    core.set_session(session).await.map_err(|e| e.to_string())?;

    // Activate the per-user session (creates dirs, opens the session store).
    runtime.activate(user_id).await?;

    activate_per_user_stores(runtime, user_id).await;

    // Persist the token so the next launch restores the session silently.
    if let Err(e) = qbz_credentials::save_oauth_token(&token) {
        log::warn!("[qbz-slint] failed to persist OAuth token: {e}");
    }

    log::info!("[qbz-slint] login complete for user {user_id}");
    Ok(SessionInfo {
        user_id,
        display_name,
        subscription,
    })
}
