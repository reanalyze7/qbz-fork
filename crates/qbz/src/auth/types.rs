//! Shared auth types.

use std::time::Duration;

pub(super) const OAUTH_TIMEOUT: Duration = Duration::from_secs(180);

/// The authenticated user, as the shell needs it.
pub struct SessionInfo {
    pub user_id: u64,
    pub display_name: String,
    pub subscription: String,
}

/// Progress milestones of the browser OAuth, reported to the login UI so
/// the screen can narrate the flow (the browser may open in the background
/// without stealing focus, and the code exchange takes a few seconds).
#[derive(Clone, Copy, Debug)]
pub enum LoginPhase {
    /// The browser was opened; waiting for the user to finish signing in
    /// and for the redirect to land on the local listener.
    WaitingForBrowser,
    /// The authorization code was captured; exchanging it for a session.
    Authenticating,
}
