use ratatui::crossterm::event::{KeyCode, KeyEvent};

use crate::tui::app::ScreenAction;

use super::state::BundleState;

impl BundleState {
    pub(super) fn handle_review_key(&mut self, key: KeyEvent) -> ScreenAction {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.scroll = self.scroll.saturating_sub(1);
                ScreenAction::Consumed
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.scroll = self.scroll.saturating_add(1);
                ScreenAction::Consumed
            }
            KeyCode::Char('p') => {
                self.open_device_picker();
                ScreenAction::Consumed
            }
            KeyCode::Enter => {
                let has_auth = self.pending.as_ref().map(|p| p.has_auth).unwrap_or(false);
                if has_auth {
                    self.auth_confirm = true;
                    ScreenAction::Consumed
                } else {
                    if let Some(p) = self.pending.as_mut() {
                        p.apply_with_auth = false;
                    }
                    ScreenAction::ImportApply
                }
            }
            KeyCode::Esc => {
                self.pending = None;
                ScreenAction::Consumed
            }
            _ => ScreenAction::Consumed,
        }
    }

    pub(super) fn handle_auth_confirm(&mut self, key: KeyEvent) -> ScreenAction {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if let Some(p) = self.pending.as_mut() {
                    p.apply_with_auth = true;
                }
                self.auth_confirm = false;
                ScreenAction::ImportApply
            }
            KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                if let Some(p) = self.pending.as_mut() {
                    p.apply_with_auth = false;
                }
                self.auth_confirm = false;
                ScreenAction::ImportApply
            }
            _ => ScreenAction::Consumed,
        }
    }
}
