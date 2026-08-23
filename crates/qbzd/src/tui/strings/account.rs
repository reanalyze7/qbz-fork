// ============================ Account (§3.1) ============================

pub const ACCOUNT_TITLE: &str = "Account";
/// In-screen section box title (distinct from the screen title in the frame).
pub const ACCOUNT_SECTION: &str = "SIGN-IN";
pub const ACCOUNT_STATUS: &str = "Status";
pub const ACCOUNT_NOT_LOGGED_IN: &str = "not logged in";
/// Offline + daemon-down: a credential file exists but was never validated —
/// NEVER fabricate an email/name (§3.1 rules).
pub const ACCOUNT_CRED_PRESENT: &str = "credential file present (not validated)";
pub const ACCOUNT_LOGIN_BROWSER: &str = "Log in with browser";
pub const ACCOUNT_PASTE_TOKEN: &str = "Paste token";
pub const ACCOUNT_LOGOUT: &str = "Log out";

pub fn account_logged_in(email: &str) -> String {
    format!("logged in as {email}")
}
pub fn account_logged_in_plan(email: &str, plan: &str) -> String {
    format!("logged in as {email} ({plan})")
}

pub const ACCOUNT_LOGOUT_CONFIRM_TITLE: &str = "Log out";
pub const ACCOUNT_LOGOUT_CONFIRM_BODY: &str =
    "Clear the Qobuz credentials on this box? If the daemon is running it will\nstop playback and wait for a new login.";
pub const CONFIRM_YN: &str = "y confirm · Esc cancel";

pub const ACCOUNT_VALIDATING: &str = "validating token with Qobuz…";

/// Suspend-and-run divergence banner (see report): the browser flow runs on the
/// plain terminal. Shown briefly before the alt-screen is left.
pub const ACCOUNT_BROWSER_HANDOFF: &str =
    "Starting browser login on the terminal below. Follow the printed URL;\nthe TUI resumes when login finishes or times out.";
