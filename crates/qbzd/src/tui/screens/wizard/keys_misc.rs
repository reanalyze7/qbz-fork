// crates/qbzd/src/tui/screens/wizard/keys_misc.rs — Test + Done step key
// handling.

use ratatui::crossterm::event::{KeyCode, KeyEvent};

use crate::tui::app::ScreenAction;

use super::state::WizardState;

impl WizardState {
    pub(super) fn keys_test(&mut self, key: KeyEvent) -> ScreenAction {
        match key.code {
            KeyCode::Enter | KeyCode::Char('t') => ScreenAction::WizardTestStart,
            KeyCode::Char('r') => {
                if self.tested {
                    ScreenAction::WizardTestPoll
                } else {
                    ScreenAction::WizardTestStart
                }
            }
            _ => ScreenAction::Consumed,
        }
    }

    pub(super) fn keys_done(&mut self, key: KeyEvent) -> ScreenAction {
        if matches!(key.code, KeyCode::Enter) {
            ScreenAction::Back
        } else {
            ScreenAction::Consumed
        }
    }
}
