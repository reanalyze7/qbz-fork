// crates/qbzd/src/cli/settings/nudge.rs — best-effort local-daemon nudge
// after a write (`login::nudge_reload`, the same ping-then-reload pattern
// `login`/`logout` already use).

use crate::paths::ProfileRoots;

/// Best-effort nudge of a LOCAL running daemon (§1.1/§1.5: these verbs always
/// target the daemon whose stores they just wrote, never `--host`/
/// `QBZD_HOST` — same reasoning as `login`/`logout`). Reads the opt-in
/// `[server] token` the same way `cli/client.rs::resolve_token` does for the
/// local target, so a token-protected daemon still gets nudged.
pub(super) fn nudge(roots: &ProfileRoots) -> bool {
    let host = crate::login::nudge_host(roots);
    let token = local_token(roots);
    crate::login::nudge_reload(&host, token.as_deref())
}

/// Three-state variant of [`nudge`] for `settings import`, which must
/// distinguish daemon-down from reload-refused (04 §5.3 step 7).
pub(super) fn nudge_outcome(roots: &ProfileRoots) -> crate::login::NudgeOutcome {
    let host = crate::login::nudge_host(roots);
    let token = local_token(roots);
    crate::login::nudge_reload_outcome(&host, token.as_deref())
}

fn local_token(roots: &ProfileRoots) -> Option<String> {
    if let Ok(t) = std::env::var("QBZD_TOKEN") {
        if !t.is_empty() {
            return Some(t);
        }
    }
    crate::config::QbzdConfig::load(&roots.config.join("qbzd.toml"))
        .ok()
        .and_then(|(c, _)| c.server.token)
        .filter(|t| !t.trim().is_empty())
}
