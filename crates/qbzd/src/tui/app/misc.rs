// crates/qbzd/src/tui/app/misc.rs — the two callbacks the event loop invokes
// after suspending the alt-screen to run a plain-terminal auth flow
// (`login_flows.rs`), once it resumes.

use crate::tui::screens::account::AuthSnapshot;
use crate::tui::screens::scrobbler::ScrobblerState;
use crate::tui::strings as s;

use super::messages::Screen;
use super::messages_worker::{Active, Overlay};
use super::state::App;

impl App {
    /// Called by the loop after it runs the suspended browser-login engine.
    pub fn after_browser_login(&mut self, result: Result<(String, Option<String>), String>) {
        match result {
            Ok((email, plan)) => {
                self.auth = AuthSnapshot {
                    logged_in: true,
                    email: Some(email.clone()),
                    plan: plan.clone(),
                    cred_file_present: true,
                };
                if let Active::Account(st) = &mut self.active {
                    st.set_auth(self.auth.clone());
                }
                self.overlay = Overlay::Result {
                    title: s::ACCOUNT_TITLE.to_string(),
                    lines: vec![s::account_logged_in(&email)],
                };
            }
            Err(e) => {
                self.overlay = Overlay::Result {
                    title: s::ACCOUNT_TITLE.to_string(),
                    lines: e.lines().map(str::to_string).collect(),
                };
            }
        }
    }

    /// Reload the Scrobbler screen's settings snapshot after a connect flow ran
    /// on the suspended plain terminal (the CLI auth wrote the store directly).
    pub fn refresh_scrobbler(&mut self) {
        if matches!(self.active_section, Screen::Scrobbler) {
            self.active = Active::Scrobbler(ScrobblerState::new(&self.roots));
        }
    }
}
