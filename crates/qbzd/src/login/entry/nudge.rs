use crate::login::error::LoginError;
use crate::login::io::{http_request_2xx, nudge_host};
use crate::paths::ProfileRoots;

/// `qbzd logout` (02 §2.2): clear the daemon-root credential file and nudge a
/// running daemon into NeedsAuth. Returns whether the daemon acknowledged the
/// reload, so the caller can pick the right success line.
pub fn logout(roots: &ProfileRoots) -> Result<bool, LoginError> {
    qbz_credentials::clear_oauth_token_at(&roots.config)
        .map_err(|e| LoginError::Failed(format!("could not clear the credential file: {e}")))?;
    let host = nudge_host(roots);
    // token: opt-in [server] token, wired by T6.
    Ok(nudge_reload(&host, None))
}

/// Three-state outcome of the ping-then-reload nudge. 04-settings-portability.md
/// §5.3 step 7 needs "daemon simply not running" (not an error) distinguished
/// from "daemon up but the reload was refused/500" (exit 1 with the restart
/// hint) — a single bool conflates them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NudgeOutcome {
    /// Ping answered and the reload returned 2xx.
    Reloaded,
    /// Ping did not answer — no daemon to nudge (never an error).
    DaemonDown,
    /// Ping answered but the reload did not return 2xx.
    ReloadRefused,
}

/// Best-effort `GET /api/ping` → `POST /api/settings/reload` against a local
/// daemon. `token` carries the opt-in `[server] token` as
/// `Authorization: Bearer` when present; T5 callers pass `None`.
pub fn nudge_reload_outcome(host: &str, token: Option<&str>) -> NudgeOutcome {
    if !http_request_2xx(host, "GET", "/api/ping", token) {
        return NudgeOutcome::DaemonDown;
    }
    if http_request_2xx(host, "POST", "/api/settings/reload", token) {
        NudgeOutcome::Reloaded
    } else {
        NudgeOutcome::ReloadRefused
    }
}

/// Boolean skin over [`nudge_reload_outcome`] for the callers that only need
/// "did a running daemon acknowledge?" (login/logout/`settings set` — they are
/// specified to work daemon-down, 02 §2.2, so any non-reload is just "the
/// daemon picks it up on next start").
pub fn nudge_reload(host: &str, token: Option<&str>) -> bool {
    nudge_reload_outcome(host, token) == NudgeOutcome::Reloaded
}
