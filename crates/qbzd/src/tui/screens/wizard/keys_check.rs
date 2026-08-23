// crates/qbzd/src/tui/screens/wizard/keys_check.rs — Welcome + Check step key
// handling, including the distro/init override select-popup editor.

use qbz_audio::{Distro, InitSystem};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

use crate::tui::app::ScreenAction;
use crate::tui::strings as s;
use crate::tui::widgets::{SelectOutcome, SelectPopup};

use super::state::WizardState;
use super::state_types::CheckField;

impl WizardState {
    pub(super) fn keys_welcome(&mut self, key: KeyEvent) -> ScreenAction {
        if matches!(key.code, KeyCode::Enter | KeyCode::Char(' ')) {
            self.advance()
        } else {
            ScreenAction::Consumed
        }
    }

    pub(super) fn keys_check(&mut self, key: KeyEvent) -> ScreenAction {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.check_focus = self.check_focus.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.check_focus = (self.check_focus + 1).min(1);
            }
            KeyCode::Enter | KeyCode::Char(' ') => self.open_check_editor(),
            _ => {}
        }
        ScreenAction::Consumed
    }

    fn open_check_editor(&mut self) {
        if self.check_focus == 0 {
            let opts: Vec<String> = Distro::ALL.iter().map(|d| d.label().to_string()).collect();
            self.check_editor = Some((
                CheckField::Distro,
                SelectPopup::new(s::WIZ_DISTRO, opts, self.distro_index, false),
            ));
        } else {
            let opts: Vec<String> = InitSystem::ALL.iter().map(|i| i.label().to_string()).collect();
            self.check_editor = Some((
                CheckField::Init,
                SelectPopup::new(s::WIZ_INIT, opts, self.init_index, false),
            ));
        }
    }

    pub(super) fn handle_check_editor(&mut self, key: KeyEvent) -> ScreenAction {
        let (field, mut popup) = self.check_editor.take().unwrap();
        match popup.handle_key(key) {
            SelectOutcome::Chosen(i) => match field {
                CheckField::Distro => self.distro_index = i,
                CheckField::Init => self.init_index = i,
            },
            SelectOutcome::Cancelled => {}
            SelectOutcome::Pending => self.check_editor = Some((field, popup)),
        }
        ScreenAction::Consumed
    }
}
