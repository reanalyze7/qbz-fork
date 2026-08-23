//! `restore_saved_session`: restore a previously saved session from the
//! encrypted token store.

use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;

use super::init::{ensure_api_initialized, is_auth_rejection};
use super::per_user::activate_per_user_stores;
use super::types::SessionInfo;

/// Restore a previously saved session from the encrypted token store
/// (keyring + AES-256-GCM file — the same store the Tauri app uses).
///
/// Returns `Ok(Some(SessionInfo))` when a saved token is valid and the
/// session is activated, `Ok(None)` when there is no token. A token that
/// exists but is explicitly rejected by Qobuz is cleared and treated as
/// `None`; on network-class failures the token is kept so the session can
/// still be restored later (offline boot, D2 recovery).
pub async fn restore_saved_session<A>(
    runtime: &Arc<AppRuntime<A>>,
) -> Result<Option<SessionInfo>, String>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    let token = match qbz_credentials::load_oauth_token() {
        Ok(Some(token)) => token,
        Ok(None) => return Ok(None),
        Err(e) => {
            log::warn!("[qbz-slint] could not read saved token: {e}");
            return Ok(None);
        }
    };
    // Same redaction registration as the browser login path — cold restore
    // must scrub bare token substrings from logs for the whole session.
    qbz_log::register_secret(token.clone());

    let core = runtime.core();
    // Same scoped handling of the gated cold bundle init as the browser
    // flow (no-op when the client already holds tokens).
    ensure_api_initialized(core).await?;

    match core.login_with_token(&token).await {
        Ok(session) => {
            let user_id = session.user_id;
            let display_name = session.display_name.clone();
            let subscription = session.subscription_label.clone();
            core.set_session(session).await.map_err(|e| e.to_string())?;
            runtime.activate(user_id).await?;
            activate_per_user_stores(runtime, user_id).await;
            log::info!("[qbz-slint] restored saved session for user {user_id}");
            Ok(Some(SessionInfo {
                user_id,
                display_name,
                subscription,
            }))
        }
        Err(e) if is_auth_rejection(&e) => {
            log::warn!("[qbz-slint] saved token rejected by Qobuz, clearing: {e}");
            let _ = qbz_credentials::clear_oauth_token();
            // D4 producer: only the explicit ineligible verdict starts the
            // grace clock; a plain 401 does not.
            if matches!(
                &e,
                qbz_core::CoreError::Api(qbz_qobuz::ApiError::IneligibleUser)
            ) {
                crate::offline_mode::subscription_mark_invalid();
            }
            Ok(None)
        }
        Err(e) => {
            // Network-class failure (offline boot, timeout, 5xx, ...): KEEP
            // the token. The login screen shows with the session intact so
            // "Start offline" / the D2 recovery banner can use it later.
            log::warn!("[qbz-slint] session restore failed, keeping saved token: {e}");
            Ok(None)
        }
    }
}
