use ratatui::crossterm::event::{KeyCode, KeyEvent};

use crate::tui::app::ScreenAction;
use crate::tui::strings as s;
use crate::tui::widgets::{InputOutcome, TextInput};

/// Snapshot of the auth state the App resolves (from `GET /api/status` when the
/// daemon runs, else from credential-file presence).
#[derive(Debug, Clone, Default)]
pub struct AuthSnapshot {
    pub logged_in: bool,
    pub email: Option<String>,
    pub plan: Option<String>,
    /// Offline + daemon-down: a credential file exists but is unvalidated.
    pub cred_file_present: bool,
}

pub struct AccountState {
    pub(super) auth: AuthSnapshot,
    pub(super) focus: usize,
    pub(super) token_input: Option<TextInput>,
    pub(super) confirm_logout: bool,
}

impl AccountState {
    pub fn new(auth: AuthSnapshot) -> Self {
        Self {
            auth,
            focus: 0,
            token_input: None,
            confirm_logout: false,
        }
    }

    pub fn set_auth(&mut self, auth: AuthSnapshot) {
        self.auth = auth;
    }

    // Account is all immediate actions — it never participates in dirty-state,
    // so the App's `active_is_dirty` short-circuits it (no `is_dirty` here).
    pub fn is_editing(&self) -> bool {
        self.token_input.is_some() || self.confirm_logout
    }

    /// The breadcrumb's level-2 node when an INLINE field edit is active. The
    /// logout confirm is a third-level modal (an overlay) — the breadcrumb
    /// underneath stays `Setup › Account`, so it returns None.
    pub fn editing_label(&self) -> Option<&'static str> {
        if self.token_input.is_some() {
            Some(s::ACCOUNT_PASTE_TOKEN)
        } else {
            None
        }
    }

    /// Action rows in focus order, depending on login state.
    pub(super) fn actions(&self) -> Vec<&'static str> {
        if self.auth.logged_in {
            vec![s::ACCOUNT_LOGOUT, s::ACCOUNT_PASTE_TOKEN]
        } else {
            vec![s::ACCOUNT_LOGIN_BROWSER, s::ACCOUNT_PASTE_TOKEN]
        }
    }

    // -------------------------- input --------------------------

    pub fn handle_key(&mut self, key: KeyEvent) -> ScreenAction {
        if let Some(mut input) = self.token_input.take() {
            match input.handle_key(key) {
                InputOutcome::Accepted => {
                    let token = input.buf.trim().to_string();
                    if token.is_empty() {
                        return ScreenAction::Consumed;
                    }
                    return ScreenAction::LoginToken(token);
                }
                InputOutcome::Cancelled => return ScreenAction::Consumed,
                InputOutcome::Pending => {
                    self.token_input = Some(input);
                    return ScreenAction::Consumed;
                }
            }
        }
        if self.confirm_logout {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    self.confirm_logout = false;
                    return ScreenAction::Logout;
                }
                KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                    self.confirm_logout = false;
                    return ScreenAction::Consumed;
                }
                _ => return ScreenAction::Consumed,
            }
        }

        let actions = self.actions();
        match key.code {
            KeyCode::Up | KeyCode::Char('k') | KeyCode::BackTab => {
                self.focus = if self.focus == 0 { actions.len() - 1 } else { self.focus - 1 };
                ScreenAction::Consumed
            }
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => {
                self.focus = (self.focus + 1) % actions.len();
                ScreenAction::Consumed
            }
            KeyCode::Enter | KeyCode::Char(' ') => match actions[self.focus] {
                s::ACCOUNT_LOGIN_BROWSER => ScreenAction::LoginBrowser,
                s::ACCOUNT_PASTE_TOKEN => {
                    self.token_input = Some(TextInput::new("", true));
                    ScreenAction::Consumed
                }
                s::ACCOUNT_LOGOUT => {
                    self.confirm_logout = true;
                    ScreenAction::Consumed
                }
                _ => ScreenAction::Consumed,
            },
            KeyCode::Esc => ScreenAction::Back,
            _ => ScreenAction::Consumed,
        }
    }
}
