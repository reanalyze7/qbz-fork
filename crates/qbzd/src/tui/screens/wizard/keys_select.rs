// crates/qbzd/src/tui/screens/wizard/keys_select.rs — Select-DACs step key
// handling, including the manual node-name input.

use ratatui::crossterm::event::{KeyCode, KeyEvent};
use std::time::Instant;

use crate::tui::app::ScreenAction;
use crate::tui::strings as s;
use crate::tui::widgets::{InputOutcome, TextInput};
use crate::tui::wizard_core;

use super::state::WizardState;

impl WizardState {
    pub(super) fn keys_select(&mut self, key: KeyEvent) -> ScreenAction {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if !self.candidates.is_empty() {
                    self.dac_focus = self.dac_focus.saturating_sub(1);
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.candidates.is_empty() {
                    self.dac_focus = (self.dac_focus + 1).min(self.candidates.len() - 1);
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                if let Some(c) = self.candidates.get_mut(self.dac_focus) {
                    c.checked = !c.checked;
                }
            }
            KeyCode::Char('m') => {
                self.manual = Some(TextInput::new(
                    self.manual_node.as_deref().unwrap_or(""),
                    false,
                ));
            }
            _ => {}
        }
        ScreenAction::Consumed
    }

    pub(super) fn handle_manual_input(&mut self, key: KeyEvent) -> ScreenAction {
        let mut input = self.manual.take().unwrap();
        match input.handle_key(key) {
            InputOutcome::Accepted => {
                let text = input.buf.trim().to_string();
                if wizard_core::validate_node_name(&text) {
                    self.manual_node = Some(text);
                } else if !text.is_empty() {
                    self.gate_note = Some((s::WIZ_MANUAL_INVALID.to_string(), Instant::now()));
                    self.manual = Some(input); // keep it open to fix
                }
            }
            InputOutcome::Cancelled => {}
            InputOutcome::Pending => self.manual = Some(input),
        }
        ScreenAction::Consumed
    }
}
