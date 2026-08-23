use ratatui::crossterm::event::KeyEvent;

use crate::tui::app::ScreenAction;
use crate::tui::widgets::SelectOutcome;

use super::fields::MAX_RATES;
use super::model::{Editor, PlaybackState};

impl PlaybackState {
    pub(super) fn handle_editor_key(&mut self, key: KeyEvent) -> ScreenAction {
        let editor = self.editor.take().unwrap();
        match editor {
            Editor::Quality(mut p) => match p.handle_key(key) {
                SelectOutcome::Chosen(i) => {
                    self.staged.quality = ["mp3", "cd", "hires", "hires_plus"]
                        .get(i)
                        .copied()
                        .unwrap_or("hires_plus")
                        .to_string();
                    ScreenAction::Consumed
                }
                SelectOutcome::Cancelled => ScreenAction::Consumed,
                SelectOutcome::Pending => {
                    self.editor = Some(Editor::Quality(p));
                    ScreenAction::Consumed
                }
            },
            Editor::MaxRate(mut p) => match p.handle_key(key) {
                SelectOutcome::Chosen(i) => {
                    self.staged.max_sample_rate = MAX_RATES.get(i).map(|(_, v)| *v).unwrap_or(None);
                    ScreenAction::Consumed
                }
                SelectOutcome::Cancelled => ScreenAction::Consumed,
                SelectOutcome::Pending => {
                    self.editor = Some(Editor::MaxRate(p));
                    ScreenAction::Consumed
                }
            },
            Editor::Retry(mut p) => match p.handle_key(key) {
                SelectOutcome::Chosen(i) => {
                    self.staged.fallback_behavior =
                        if i == 1 { "always_skip" } else { "always_fallback" }.to_string();
                    ScreenAction::Consumed
                }
                SelectOutcome::Cancelled => ScreenAction::Consumed,
                SelectOutcome::Pending => {
                    self.editor = Some(Editor::Retry(p));
                    ScreenAction::Consumed
                }
            },
        }
    }
}
