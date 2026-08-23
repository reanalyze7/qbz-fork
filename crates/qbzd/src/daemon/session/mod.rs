mod activate;
mod retry;

use std::sync::{Arc, Mutex};

use qbz_core::CoreError;
use qbz_models::UserSession;

use crate::config::QbzdConfig;
use crate::state::{AuthState, DaemonShared, LatchedErrors};

pub(crate) use activate::restore_activate;
pub(crate) use retry::spawn_auth_retry;

/// Fresh shared state. Starts in `Restoring` — credential restore drives the
/// terminal transition to `LoggedIn` or `NeedsAuth` (§6.2 diagram).
pub(super) fn new_shared(cfg: &QbzdConfig) -> Arc<Mutex<DaemonShared>> {
    let _ = cfg; // reserved: premute/mpris defaults wire in with later tasks.
    Arc::new(Mutex::new(DaemonShared {
        auth: AuthState::Restoring,
        user_id: None,
        subscription: None,
        last_errors: LatchedErrors::default(),
        driver_last_tick: None,
        muted: false,
        premute_volume: 1.0,
        started_at: std::time::Instant::now(),
        startup_warnings: 0,
        credential_fingerprint: None,
        network_online: std::sync::atomic::AtomicBool::new(true),
    }))
}

/// Enter NeedsAuth. `err = None` = no saved credentials at all (the common
/// first-run case); `Some(e)` = an explicit auth rejection just cleared the
/// token. Either way the daemon STAYS UP (§6.2) and names the fix.
pub(crate) fn set_needs_auth(shared: &Arc<Mutex<DaemonShared>>, err: Option<CoreError>) {
    if let Ok(mut s) = shared.lock() {
        s.auth = AuthState::NeedsAuth;
        s.user_id = None;
        s.subscription = None;
        // T11: NeedsAuth has no applied token by definition.
        s.credential_fingerprint = None;
    }
    match err {
        None => log::info!("Not logged in — run 'qbzd setup' (or 'qbzd login')"),
        Some(e) => {
            log::warn!("Qobuz rejected the saved session ({e}) — run 'qbzd login' to re-authenticate")
        }
    }
}

/// Enter LoggedIn (Ready). Records the user id + subscription label for
/// `/api/status` (T6). The auth token itself is never stored here — it is a
/// registered secret and lives only in the credential file.
pub(super) fn set_logged_in(shared: &Arc<Mutex<DaemonShared>>, session: &UserSession) {
    if let Ok(mut s) = shared.lock() {
        s.auth = AuthState::LoggedIn;
        s.user_id = Some(session.user_id);
        s.subscription = Some(session.subscription_label.clone());
    }
    log::info!(
        "Logged in (user {}, subscription '{}')",
        session.user_id,
        session.subscription_label
    );
}

/// Latch the "saved token is present but undecryptable" case. Reachable when a
/// token was written under a key this process cannot derive — e.g. a login that
/// ran in a graphical session against a build that still mixed the XDG portal
/// secret in, now read by an init-started daemon with no session bus. The
/// credential store migrates that token itself where it can (a daemon that CAN
/// reach the portal rewrites it portal-free); when it cannot, the only exit is a
/// fresh login, so say exactly that instead of a bare "not logged in".
pub(crate) fn latch_undecryptable_token(shared: &Arc<Mutex<DaemonShared>>) {
    if let Ok(mut s) = shared.lock() {
        s.last_errors.auth = Some(
            "the saved token could not be decrypted by this daemon — run 'qbzd login' to re-authenticate".into(),
        );
    }
    log::warn!(
        "saved token present but undecryptable — run 'qbzd login' to re-authenticate"
    );
}

/// Latch an auth error so a `status` call remains diagnosable after the fact
/// (§9.4 — drain-once channels alone cannot answer "why did the music stop?").
pub(crate) fn latch_auth_error(shared: &Arc<Mutex<DaemonShared>>, e: &CoreError) {
    if let Ok(mut s) = shared.lock() {
        s.last_errors.auth = Some(format!("token rejected by Qobuz — cleared ({e})"));
    }
}

/// True ONLY for an explicit auth rejection from Qobuz — a 401 on the token
/// login (`AuthenticationError`) or an ineligible-account verdict. Network
/// failures, offline gate, 5xx, rate limiting and parse errors all return false
/// so the saved token is KEPT (mirrors crates/qbz/src/auth.rs:215-230; the
/// taxonomy — not the variant list — is the normative part).
pub(crate) fn is_auth_rejection(error: &CoreError) -> bool {
    matches!(
        error,
        CoreError::Api(
            qbz_qobuz::ApiError::AuthenticationError(_) | qbz_qobuz::ApiError::IneligibleUser
        )
    )
}
