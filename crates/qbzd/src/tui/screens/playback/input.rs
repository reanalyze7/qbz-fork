use ratatui::crossterm::event::{KeyCode, KeyEvent};

use crate::tui::app::ScreenAction;
use crate::tui::strings as s;
use crate::tui::widgets::SelectPopup;

use super::fields::{row_state, visible_fields, PField, MAX_RATES};
use super::model::{Editor, PlaybackState};

impl PlaybackState {
    // -------------------------- input --------------------------

    pub fn handle_key(&mut self, key: KeyEvent) -> ScreenAction {
        if self.editor.is_some() {
            return self.handle_editor_key(key);
        }
        let fields = visible_fields(&self.staged);
        if self.focus >= fields.len() {
            self.focus = fields.len().saturating_sub(1);
        }
        match key.code {
            KeyCode::Up | KeyCode::Char('k') | KeyCode::BackTab => {
                if self.focus == 0 {
                    self.focus = fields.len().saturating_sub(1);
                } else {
                    self.focus -= 1;
                }
                ScreenAction::Consumed
            }
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => {
                if !fields.is_empty() {
                    self.focus = (self.focus + 1) % fields.len();
                }
                ScreenAction::Consumed
            }
            KeyCode::Char('s') => ScreenAction::Save,
            KeyCode::Enter | KeyCode::Char(' ') => {
                if let Some(f) = fields.get(self.focus).copied() {
                    self.activate(f);
                }
                ScreenAction::Consumed
            }
            KeyCode::Esc => ScreenAction::Back,
            _ => ScreenAction::Consumed,
        }
    }

    fn activate(&mut self, field: PField) {
        let (_, enabled, _) = row_state(field, &self.staged);
        if !enabled {
            return;
        }
        match field {
            PField::Quality => {
                let opts = vec![
                    s::Q_MP3.to_string(),
                    s::Q_CD.to_string(),
                    s::Q_HIRES.to_string(),
                    s::Q_HIRES_PLUS.to_string(),
                ];
                let sel = match self.staged.quality.as_str() {
                    "mp3" => 0,
                    "cd" => 1,
                    "hires" => 2,
                    _ => 3,
                };
                self.editor = Some(Editor::Quality(SelectPopup::new(s::P_QUALITY, opts, sel, false)));
            }
            PField::MaxRate => {
                let opts: Vec<String> = MAX_RATES.iter().map(|(l, _)| l.to_string()).collect();
                let sel = MAX_RATES
                    .iter()
                    .position(|(_, v)| *v == self.staged.max_sample_rate)
                    .unwrap_or(0);
                self.editor = Some(Editor::MaxRate(SelectPopup::new(s::P_MAX_RATE, opts, sel, false)));
            }
            PField::RetryFail => {
                let opts = vec![s::RETRY_FALLBACK.to_string(), s::RETRY_SKIP.to_string()];
                let sel = if self.staged.fallback_behavior == "always_skip" { 1 } else { 0 };
                self.editor = Some(Editor::Retry(SelectPopup::new(s::P_RETRY_FAIL, opts, sel, false)));
            }
            PField::Limit => self.staged.limit_to_device ^= true,
            PField::AllowFallback => self.staged.allow_fallback ^= true,
            PField::Continue => {
                // §3.3.1: infinite (radio) is preserved until toggled; the first
                // toggle from infinite lands on off (track_only).
                self.staged.autoplay = match self.staged.autoplay.as_str() {
                    "continue" => "track_only",
                    _ => "continue", // track_only or infinite → continue
                }
                .to_string();
            }
            PField::Gapless => self.staged.gapless ^= true,
            PField::Restore => self.staged.restore_session ^= true,
            PField::Resume => self.staged.resume_position ^= true,
            PField::Mpris => self.staged.mpris ^= true,
        }
    }
}
