use std::time::Instant;

use qbz_models::UserSession;

use super::LOGIN_DEADLINE;
use crate::login::error::LoginError;
use crate::login::io::{bind_login_listener, capture_callback, exchange_code, finalize, read_app_id};
use crate::login::io::build_login_runtime;
use crate::login::parsing::{build_oauth_url, gen_nonce, resolve_callback_host};
use crate::paths::ProfileRoots;

/// Path 1 (02 §2.2): system-browser OAuth on a one-shot, nonce-bound, ephemeral
/// listener. FB1 (owner feedback, post-smoke): the common real-world case is
/// configuring the daemon headless over SSH from another machine on the LAN,
/// so the callback host defaults to that LAN-reachable address — not
/// loopback-only — via [`resolve_callback_host`]. `callback_host = Some(ip)`
/// keeps its old explicit-override meaning; `None` now auto-detects from
/// `SSH_CONNECTION` before falling back to `127.0.0.1`.
pub async fn login_browser(
    roots: &ProfileRoots,
    callback_host: Option<String>,
) -> Result<UserSession, LoginError> {
    let runtime = build_login_runtime().await?;
    let app_id = read_app_id(&runtime).await?;

    // D6: an EPHEMERAL port on its own listener — never the control-API port.
    let ssh_connection = std::env::var("SSH_CONNECTION").ok();
    let (redirect_host, auto_detected) =
        resolve_callback_host(callback_host.as_deref(), ssh_connection.as_deref());

    let listener = bind_login_listener(&redirect_host)?;
    let port = listener
        .local_addr()
        .map_err(|e| LoginError::Failed(e.to_string()))?
        .port();

    let nonce = gen_nonce();
    let url = build_oauth_url(&app_id, &redirect_host, port, &nonce);

    // URL first, ALWAYS — the auto-detect note follows it.
    println!("Opening your browser to sign in to Qobuz.");
    println!("If it does not open, paste this URL into a browser:\n  {url}\n");
    if auto_detected {
        println!("{}", crate::cli::copy::login_ssh_detected());
    }
    if let Err(e) = open::that(&url) {
        // Headless boxes have no browser — not fatal; the listener still waits
        // and the printed URL (already shown above) is what the operator
        // forwards/opens from another device. Never an error-looking line.
        log::debug!("could not open a browser automatically: {e}");
        println!("{}", crate::cli::copy::login_browser_open_failed());
    }

    let nonce_owned = nonce.clone();
    let deadline = Instant::now() + LOGIN_DEADLINE;
    let captured = tokio::task::spawn_blocking(move || {
        capture_callback(listener, &nonce_owned, deadline)
    })
    .await
    .map_err(|e| LoginError::Failed(format!("login listener task panicked: {e}")))?
    .map_err(|e| LoginError::Failed(format!("login listener I/O error: {e}")))?;

    let code = captured.ok_or(LoginError::Timeout(port))?;
    let session = exchange_code(&runtime, &code).await?;
    finalize(roots, &session)?;
    Ok(session)
}
