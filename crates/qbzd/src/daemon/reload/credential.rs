use std::sync::{Arc, Mutex};

use qbz_app::shell::AppRuntime;
use qbz_app::playback_driver;

use crate::adapter::DaemonAdapter;
use crate::paths::ProfileRoots;
use crate::state::{AuthState, DaemonShared};

use crate::daemon::session::{
    is_auth_rejection, latch_auth_error, restore_activate, set_needs_auth,
};

/// What the freshly-read credential file implies for the live session — pure
/// decision, unit-tested with no IO/network; [`reload_credentials`] just
/// executes whichever variant this returns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CredentialAction {
    /// The file matches what's already applied (or both are absent/NeedsAuth
    /// already) — no network call, no state churn on an unrelated nudge.
    NoOp,
    /// The file is now empty but the daemon thinks it's logged in — tear the
    /// session down (mirrors what `qbzd logout` does to a running daemon,
    /// 02 §2.2: "QConnect session torn down, playback stopped").
    EnterNeedsAuth,
    /// A token is on disk that is not the one currently applied (fresh login
    /// out of NeedsAuth, a retry while Restoring, or an account switch) —
    /// validate and activate it.
    Apply(String),
}

pub(crate) fn decide_credential_action(
    current_auth: AuthState,
    current_fingerprint: Option<u64>,
    file_token: Option<String>,
) -> CredentialAction {
    match file_token {
        None => {
            if current_auth == AuthState::NeedsAuth {
                CredentialAction::NoOp
            } else {
                CredentialAction::EnterNeedsAuth
            }
        }
        Some(token) => {
            let fp = crate::state::token_fingerprint(&token);
            if current_auth == AuthState::LoggedIn && current_fingerprint == Some(fp) {
                CredentialAction::NoOp
            } else {
                CredentialAction::Apply(token)
            }
        }
    }
}

/// Re-read the credential file and reconcile the live session against it (02
/// §3.3.17: "absent → NeedsAuth transition; new → session restore"; taxonomy
/// shared with boot, §6.2). `qconnect` is `None` only in the brief boot window
/// before step 12 populates the cell — the teardown branch just skips the
/// QConnect disconnect then (there is no session for it to hold yet).
pub(crate) async fn reload_credentials(
    runtime: &Arc<AppRuntime<DaemonAdapter>>,
    shared: &Arc<Mutex<DaemonShared>>,
    roots: &ProfileRoots,
) {
    let file_token = match qbz_credentials::load_oauth_token_at(&roots.config) {
        Ok(t) => t,
        Err(e) => {
            log::warn!("[reload] could not read the credential file: {e}");
            return;
        }
    };
    let (current_auth, current_fp) = match shared.lock() {
        Ok(s) => (s.auth, s.credential_fingerprint),
        Err(_) => return,
    };

    match decide_credential_action(current_auth, current_fp, file_token) {
        CredentialAction::NoOp => {}
        CredentialAction::EnterNeedsAuth => {
            log::info!("[reload] credential file cleared — tearing the session down (NeedsAuth)");
            let _ = runtime.core().stop();
            let _ = runtime.core().logout().await;
            let _ = runtime.deactivate().await;
            set_needs_auth(shared, None);
        }
        CredentialAction::Apply(token) => {
            qbz_log::register_secret(token.clone());
            match runtime.core().login_with_token(&token).await {
                Ok(session) => {
                    match restore_activate(runtime, shared, roots, session, &token).await {
                        Ok(()) => {
                            playback_driver::restore_session_paused(runtime.as_ref()).await;
                        }
                        Err(e) => log::warn!("[reload] session activation failed: {e}"),
                    }
                }
                Err(e) if is_auth_rejection(&e) => {
                    let _ = qbz_credentials::clear_oauth_token_at(&roots.config);
                    latch_auth_error(shared, &e);
                    set_needs_auth(shared, Some(e));
                }
                Err(e) => {
                    log::warn!("[reload] session restore deferred (network-class): {e}");
                    // 01 §9.3: real network-class outcome — latch it false.
                    if let Ok(s) = shared.lock() {
                        s.set_network_online(false);
                    }
                }
            }
        }
    }
}
