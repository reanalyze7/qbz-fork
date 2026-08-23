use qbz_models::UserSession;

/// The 300 s browser-login timeout (02 §2.2). `port` is the ephemeral port the
/// one-shot listener bound; it is interpolated into both halves of the
/// `ssh -L <port>:localhost:<port>` forward so a headless operator can tunnel the
/// exact port the redirect will target.
pub fn login_timeout(port: u16) -> String {
    format!(
        "error: no OAuth redirect received within 300 s
  → headless box? forward the port:  ssh -L {port}:localhost:{port} pi@kitchen-pi
    then open the login URL in this machine's browser
  → or paste the redirect URL:       qbzd login --paste
  → or inject a token directly:      qbzd login --token <user_auth_token>"
    )
}

/// FB1 (owner feedback, post-smoke): printed once, after the URL, when
/// `SSH_CONNECTION` auto-detected the LAN callback host. The common real case
/// is configuring the daemon headless over SSH from another machine on the
/// LAN, so the operator should know the link isn't loopback-only.
pub fn login_ssh_detected() -> &'static str {
    "detected SSH session — the login link works from any browser on your network"
}

/// FB1: `open::that` failing (e.g. a headless box with no browser) is never an
/// error — the URL is always printed above already. One unobtrusive note, not
/// an `error: ...`-shaped line.
pub fn login_browser_open_failed() -> &'static str {
    "could not open a local browser — use the URL above from another device"
}

/// Human success line for `qbzd login` (02 §2.2):
/// `logged in as user@example.com (studio) — user id 1234567`.
pub fn login_success(session: &UserSession) -> String {
    format!(
        "logged in as {} ({}) — user id {}",
        session.email, session.subscription_label, session.user_id
    )
}

/// Human success line for `qbzd logout` (02 §2.2). The daemon-up form names the
/// resulting NeedsAuth state so the operator knows playback stopped; the
/// daemon-down form is terse because there is nothing running to transition.
pub fn logout_success(daemon_nudged: bool) -> String {
    if daemon_nudged {
        "logged out — daemon is now in needs-auth state".to_string()
    } else {
        "logged out".to_string()
    }
}
