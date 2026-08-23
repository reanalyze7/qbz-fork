use ratatui::crossterm::event::{KeyCode, KeyEvent};

use crate::tui::app::ScreenAction;
use crate::tui::widgets::{InputOutcome, TextInput};

use super::state::{NField, NetworkState, FIELDS};

impl NetworkState {
    // -------------------------- input --------------------------

    pub fn handle_key(&mut self, key: KeyEvent) -> ScreenAction {
        if let Some((field, mut input)) = self.editor.take() {
            match input.handle_key(key) {
                InputOutcome::Accepted => {
                    let v = input.buf.trim().to_string();
                    match field {
                        NField::Bind => self.staged.bind = v,
                        NField::Port => self.staged.port = v,
                        NField::Token => self.staged.token = input.buf.clone(),
                    }
                }
                InputOutcome::Cancelled => {}
                InputOutcome::Pending => self.editor = Some((field, input)),
            }
            return ScreenAction::Consumed;
        }
        match key.code {
            KeyCode::Up | KeyCode::Char('k') | KeyCode::BackTab => {
                self.focus = if self.focus == 0 { FIELDS.len() - 1 } else { self.focus - 1 };
                ScreenAction::Consumed
            }
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => {
                self.focus = (self.focus + 1) % FIELDS.len();
                ScreenAction::Consumed
            }
            KeyCode::Char('s') => ScreenAction::Save,
            KeyCode::Enter => {
                let field = FIELDS[self.focus];
                let (initial, masked) = match field {
                    NField::Bind => (self.staged.bind.clone(), false),
                    NField::Port => (self.staged.port.clone(), false),
                    NField::Token => (self.staged.token.clone(), true),
                };
                self.editor = Some((field, TextInput::new(&initial, masked)));
                ScreenAction::Consumed
            }
            KeyCode::Esc => ScreenAction::Back,
            _ => ScreenAction::Consumed,
        }
    }
}
