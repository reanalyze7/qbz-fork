use std::sync::{Arc, Mutex};

use qbz_app::shell::AppRuntime;
use qbz_models::UserSession;

use crate::adapter::DaemonAdapter;
use crate::paths::ProfileRoots;
use crate::state::DaemonShared;

use super::set_logged_in;

/// Activate the per-user session against DAEMON paths (§8.1-9): inject the
/// session into the core, then `activate_at` the runtime with per-user daemon
/// data/cache directories — never the desktop `UserDataPaths`.
pub(crate) async fn restore_activate(
    runtime: &Arc<AppRuntime<DaemonAdapter>>,
    shared: &Arc<Mutex<DaemonShared>>,
    roots: &ProfileRoots,
    session: UserSession,
    token: &str,
) -> Result<(), String> {
    runtime
        .core()
        .set_session(session.clone())
        .await
        .map_err(|e| e.to_string())?;
    runtime
        .activate_at(
            session.user_id,
            &roots.data.join(format!("users/{}", session.user_id)),
            &roots.cache.join(format!("users/{}", session.user_id)),
        )
        .await?;
    set_logged_in(shared, &session);
    // T11: remember which token this activation applied so a later
    // `POST /api/settings/reload` can tell "same token" from "new token" and
    // skip a redundant re-login on every unrelated settings nudge.
    if let Ok(mut s) = shared.lock() {
        s.credential_fingerprint = Some(crate::state::token_fingerprint(token));
        // 01 §9.3: a real login/restore success (boot, background auth-retry,
        // or a reload's credential re-validation — every caller of this fn)
        // means the network is reachable — latch `network.online` back true.
        s.set_network_online(true);
    }
    Ok(())
}
