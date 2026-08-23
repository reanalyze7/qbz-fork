// ============================ public entry points ============================
mod browser;
mod nudge;

pub use browser::login_browser;
pub use nudge::{logout, nudge_reload, nudge_reload_outcome, NudgeOutcome};

use qbz_models::UserSession;

use super::error::LoginError;
use super::io::{build_login_runtime, exchange_code, finalize, read_app_id, read_stdin_line};
use super::parsing::{build_oauth_url, code_from_paste, gen_nonce};
use crate::paths::ProfileRoots;

/// Browser-login deadline (02 §2.2). The desktop uses 180 s; the daemon spec
/// pins 300 s because a headless operator may need to forward the port first.
pub(super) const LOGIN_DEADLINE: std::time::Duration = std::time::Duration::from_secs(300);

/// Cosmetic redirect port for the `--paste` flow. Nothing binds it — the browser
/// lands on a connection error and the operator copies the URL out of the address
/// bar — so the value only needs to be a syntactically valid, unprivileged port.
const PASTE_REDIRECT_PORT: u16 = 43717;

/// Live-validate a raw `user_auth_token` via `login_with_token` BEFORE it is
/// ever persisted. The returned session is the source of truth for the user id
/// and plan. Registers the token as a redaction secret first, so no log line
/// that might carry it (in the client or elsewhere) can leak it.
///
/// T12 (settings import) and T13 (setup TUI Account screen) reuse this.
pub async fn validate_token(token: &str) -> Result<UserSession, LoginError> {
    // §6.3: register before any log line can carry the token.
    qbz_log::register_secret(token.to_string());
    let runtime = build_login_runtime().await?;
    runtime
        .core()
        .login_with_token(token)
        .await
        .map_err(super::error::map_core_err)
}

/// Path 2 (02 §2.2): print the authorize URL, read the redirect URL (or a bare
/// code) back from stdin. No listener binds — useful when the browser cannot
/// reach this machine at all. A pasted redirect URL carries the nonce in its
/// path, so it is validated (leniently); a bare code is accepted as-is
/// (explicit operator action).
pub async fn login_paste(roots: &ProfileRoots) -> Result<UserSession, LoginError> {
    let runtime = build_login_runtime().await?;
    let app_id = read_app_id(&runtime).await?;
    let nonce = gen_nonce();
    let url = build_oauth_url(&app_id, "127.0.0.1", PASTE_REDIRECT_PORT, &nonce);

    println!("Open this URL in a browser and sign in to Qobuz:\n  {url}\n");
    println!("Your browser will land on a page that fails to load — that is expected.");
    print!("Paste the full redirect URL (or just the code) here: ");
    let _ = std::io::Write::flush(&mut std::io::stdout());

    let line = read_stdin_line()?;
    let code = code_from_paste(line.trim(), &nonce).ok_or_else(|| {
        LoginError::Failed(
            "could not find an authorization code in the pasted input\n  \
             → paste the full redirect URL from the browser address bar\n  \
             → or inject a token directly:      qbzd login --token <user_auth_token>"
                .to_string(),
        )
    })?;

    let session = exchange_code(&runtime, &code).await?;
    finalize(roots, &session)?;
    Ok(session)
}

/// Path 3 (02 §2.2): a directly-injected `user_auth_token`. Validated live, then
/// persisted.
pub async fn login_with_token_arg(
    roots: &ProfileRoots,
    token: &str,
) -> Result<UserSession, LoginError> {
    let session = validate_token(token).await?;
    finalize(roots, &session)?;
    Ok(session)
}
