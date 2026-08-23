use ratatui::crossterm::event::{KeyCode, KeyEvent};

use super::field::mask;

// ============================ line input ============================

#[derive(Debug, Clone, Default)]
pub struct TextInput {
    pub buf: String,
    pub masked: bool,
}

pub enum InputOutcome {
    Pending,
    Accepted,
    Cancelled,
}

impl TextInput {
    pub fn new(initial: &str, masked: bool) -> Self {
        Self { buf: initial.to_string(), masked }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> InputOutcome {
        match key.code {
            KeyCode::Enter => InputOutcome::Accepted,
            KeyCode::Esc => InputOutcome::Cancelled,
            KeyCode::Backspace => {
                self.buf.pop();
                InputOutcome::Pending
            }
            KeyCode::Char(c) => {
                self.buf.push(c);
                InputOutcome::Pending
            }
            _ => InputOutcome::Pending,
        }
    }

    pub fn display(&self) -> String {
        if self.masked {
            mask(&self.buf)
        } else {
            self.buf.clone()
        }
    }
}
