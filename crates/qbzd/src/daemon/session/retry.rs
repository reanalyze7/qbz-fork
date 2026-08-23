use std::sync::{Arc, Mutex};

use qbz_app::shell::AppRuntime;

use crate::adapter::DaemonAdapter;
use crate::paths::ProfileRoots;
use crate::state::{AuthState, DaemonShared};
use tokio::task::JoinHandle;

use super::{is_auth_rejection, latch_auth_error, restore_activate, set_needs_auth};

/// Background retry for a network-class restore failure (§6.2: stay in the
/// authenticating state, KEEP the token, retry with backoff). On success the
/// session activates; on a now-explicit auth rejection the token is cleared and
/// the daemon drops to NeedsAuth; if the whole schedule sees only network-class
/// failures the token is KEPT and the daemon surfaces NeedsAuth so it stays
/// diagnosable and a later `qbzd login` / settings reload can retry.
pub(crate) fn spawn_auth_retry(
    runtime: Arc<AppRuntime<DaemonAdapter>>,
    shared: Arc<Mutex<DaemonShared>>,
    roots: ProfileRoots,
) -> JoinHandle<()> {
    const SCHEDULE_SECS: [u64; 4] = [2, 5, 15, 30];
    tokio::spawn(async move {
        let token = match qbz_credentials::load_oauth_token_at(&roots.config) {
            Ok(Some(t)) => t,
            _ => return, // token vanished (concurrent logout) — nothing to retry.
        };
        for (i, delay) in SCHEDULE_SECS.iter().enumerate() {
            tokio::time::sleep(std::time::Duration::from_secs(*delay)).await;
            log::info!("session restore retry {}/{}", i + 1, SCHEDULE_SECS.len());
            match runtime.core().login_with_token(&token).await {
                Ok(session) => {
                    if let Err(e) =
                        restore_activate(&runtime, &shared, &roots, session, &token).await
                    {
                        log::warn!("session activation after retry failed: {e}");
                    }
                    return;
                }
                Err(e) if is_auth_rejection(&e) => {
                    let _ = qbz_credentials::clear_oauth_token_at(&roots.config);
                    latch_auth_error(&shared, &e);
                    set_needs_auth(&shared, Some(e));
                    return;
                }
                Err(e) => {
                    log::warn!("session restore retry {} failed (network-class): {e}", i + 1);
                    // 01 §9.3: latch `network.online` false on every real
                    // network-class outcome, not just the first.
                    if let Ok(s) = shared.lock() {
                        s.set_network_online(false);
                    }
                }
            }
        }
        // Schedule exhausted with only network-class failures: KEEP the token,
        // surface NeedsAuth, latch the reason for `qbzd status`.
        if let Ok(mut s) = shared.lock() {
            s.auth = AuthState::NeedsAuth;
            s.set_network_online(false);
            s.last_errors.auth = Some(
                "could not reach Qobuz to restore the saved session — token kept, retry with 'qbzd login' or 'qbzd settings reload'".into(),
            );
        }
        log::warn!(
            "session restore gave up after {} network-class attempts — token KEPT",
            SCHEDULE_SECS.len()
        );
    })
}
