// crates/qbzd/src/tui/screens/wizard/keys.rs — the top-level key dispatch and
// the step-advance/retreat state machine.

use std::time::Instant;

use ratatui::crossterm::event::{KeyCode, KeyEvent};

use crate::tui::app::ScreenAction;
use crate::tui::strings as s;

use super::state::WizardState;
use super::step::{next_step, prev_step, WStep};

impl WizardState {
    pub fn handle_key(&mut self, key: KeyEvent) -> ScreenAction {
        // Open editors own the keyboard.
        if self.check_editor.is_some() {
            return self.handle_check_editor(key);
        }
        if self.manual.is_some() {
            return self.handle_manual_input(key);
        }

        // Horizontal step navigation is uniform across steps (the shell routes
        // ←/→ to the wizard). Right advances (per-step gated); Left goes back.
        match key.code {
            KeyCode::Right => return self.advance(),
            KeyCode::Left => return self.retreat(),
            KeyCode::Esc => return self.on_escape(),
            _ => {}
        }

        match self.step {
            WStep::Welcome => self.keys_welcome(key),
            WStep::Check => self.keys_check(key),
            WStep::SelectDacs => self.keys_select(key),
            WStep::Review => self.keys_review(key),
            WStep::Test => self.keys_test(key),
            WStep::Done => self.keys_done(key),
        }
    }

    /// Esc: leave outright on the terminal steps (nothing staged), else ask to
    /// abandon (the middle steps hold transient selections).
    pub(super) fn on_escape(&mut self) -> ScreenAction {
        match self.step {
            WStep::Welcome | WStep::Done => ScreenAction::Back,
            _ => ScreenAction::WizardAbandon,
        }
    }

    /// Advance to the next step, kicking the worker the new step needs. Gated on
    /// Select-DACs (needs a selection).
    pub(super) fn advance(&mut self) -> ScreenAction {
        match self.step {
            WStep::Welcome => {
                self.step = WStep::Check;
                self.sample_host();
                ScreenAction::WizardProbeHealth
            }
            WStep::Check => {
                self.step = WStep::SelectDacs;
                if self.detected {
                    ScreenAction::Consumed // already enumerated once — keep it
                } else {
                    self.detecting = true;
                    ScreenAction::WizardDetect
                }
            }
            WStep::SelectDacs => {
                if !self.has_selection() {
                    self.gate_note = Some((s::WIZ_SELECT_GATE.to_string(), Instant::now()));
                    return ScreenAction::Consumed;
                }
                self.step = WStep::Review;
                ScreenAction::WizardGenConfigs(self.checked_dacs())
            }
            // Review → Test and Test → Done are plain linear advances (no worker),
            // so they follow the pure step-transition table directly.
            WStep::Review | WStep::Test => {
                if let Some(next) = next_step(self.step) {
                    self.step = next;
                }
                ScreenAction::Consumed
            }
            WStep::Done => ScreenAction::Back,
        }
    }

    /// Back to the previous step (no re-fetch — the state is kept).
    pub(super) fn retreat(&mut self) -> ScreenAction {
        if let Some(prev) = prev_step(self.step) {
            self.step = prev;
        }
        ScreenAction::Consumed
    }
}
