// crates/qbzd/src/tui/screens/wizard/keys_review.rs — Review step scrolling /
// focus-navigation key handling (the copy/save actions live in
// `keys_copy.rs`).

use ratatui::crossterm::event::{KeyCode, KeyEvent};

use crate::tui::app::ScreenAction;

use super::draw_review::block_line_count;
use super::state::WizardState;

impl WizardState {
    pub(super) fn keys_review(&mut self, key: KeyEvent) -> ScreenAction {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.review_focus = self.review_focus.saturating_sub(1);
                self.follow_focus();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.configs.is_empty() {
                    self.review_focus = (self.review_focus + 1).min(self.configs.len() - 1);
                }
                self.follow_focus();
            }
            KeyCode::PageUp => self.review_scroll = self.review_scroll.saturating_sub(8),
            KeyCode::PageDown => {
                self.review_scroll = self.review_scroll.saturating_add(8).min(self.max_review_scroll());
            }
            KeyCode::Char('c') => self.copy_focused_block(),
            KeyCode::Char('C') => self.copy_all_blocks(),
            KeyCode::Char('w') => self.write_focused_block(),
            _ => {}
        }
        ScreenAction::Consumed
    }

    /// Scroll so the focused block's header is at the top of the viewport.
    fn follow_focus(&mut self) {
        let mut line: u16 = 0;
        for (i, cfg) in self.configs.iter().enumerate() {
            if i == self.review_focus {
                break;
            }
            line = line.saturating_add(block_line_count(&cfg.data));
        }
        self.review_scroll = line.min(self.max_review_scroll());
    }

    /// Total rendered lines in the Review body (the backup-hint line + every
    /// block) — the ceiling `review_scroll` is clamped against so PgDn/↓ can
    /// never scroll past the last block into blank space.
    pub(super) fn review_content_lines(&self) -> u16 {
        let mut total: u16 = 1; // WIZ_BACKUP_HINT
        for cfg in &self.configs {
            total = total.saturating_add(block_line_count(&cfg.data));
        }
        total
    }

    /// The highest `review_scroll` that still shows the last content line.
    pub(super) fn max_review_scroll(&self) -> u16 {
        self.review_content_lines().saturating_sub(1)
    }
}
