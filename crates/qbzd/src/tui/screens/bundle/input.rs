use ratatui::crossterm::event::{KeyCode, KeyEvent};

use crate::tui::app::ScreenAction;
use crate::tui::widgets::{InputOutcome, TextInput};

use super::state::{BField, BundleState, Editor, FIELDS};

impl BundleState {
    // -------------------------- input --------------------------

    pub fn handle_key(&mut self, key: KeyEvent) -> ScreenAction {
        // Overlays first.
        if self.device_picker.is_some() {
            return self.handle_picker_key(key);
        }
        if self.auth_confirm {
            return self.handle_auth_confirm(key);
        }
        if self.pending.is_some() {
            return self.handle_review_key(key);
        }
        if let Some(editor) = self.editor.take() {
            return self.handle_editor_key(editor, key);
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
            KeyCode::Enter | KeyCode::Char(' ') => self.activate(),
            KeyCode::Esc => ScreenAction::Back,
            _ => ScreenAction::Consumed,
        }
    }

    pub(super) fn activate(&mut self) -> ScreenAction {
        match FIELDS[self.focus] {
            BField::ImportPath => {
                self.editor = Some(Editor::ImportPath(TextInput::new(&self.import_path, false)));
                ScreenAction::Consumed
            }
            BField::Review => {
                if self.import_path.trim().is_empty() {
                    ScreenAction::Consumed
                } else {
                    ScreenAction::ImportPlan(self.import_path.trim().to_string())
                }
            }
            BField::ExportDest => {
                self.editor = Some(Editor::ExportDest(TextInput::new(&self.export_dest, false)));
                ScreenAction::Consumed
            }
            BField::IncludeAuth => {
                self.include_auth ^= true;
                ScreenAction::Consumed
            }
            BField::Export => {
                ScreenAction::Export {
                    dest: self.export_dest.clone(),
                    include_auth: self.include_auth,
                }
            }
        }
    }

    fn handle_editor_key(&mut self, editor: Editor, key: KeyEvent) -> ScreenAction {
        match editor {
            Editor::ImportPath(mut input) => match input.handle_key(key) {
                InputOutcome::Accepted => {
                    self.import_path = input.buf.trim().to_string();
                    ScreenAction::Consumed
                }
                InputOutcome::Cancelled => ScreenAction::Consumed,
                InputOutcome::Pending => {
                    self.editor = Some(Editor::ImportPath(input));
                    ScreenAction::Consumed
                }
            },
            Editor::ExportDest(mut input) => match input.handle_key(key) {
                InputOutcome::Accepted => {
                    self.export_dest = input.buf.trim().to_string();
                    ScreenAction::Consumed
                }
                InputOutcome::Cancelled => ScreenAction::Consumed,
                InputOutcome::Pending => {
                    self.editor = Some(Editor::ExportDest(input));
                    ScreenAction::Consumed
                }
            },
        }
    }

}
