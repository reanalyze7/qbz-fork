// crates/qbzd/src/tui/screens/wizard/draw.rs — the draw dispatch + overlays,
// and the Welcome step's render. Per-step render code lives in sibling
// `draw_*.rs` files.

use std::time::Duration;

use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::tui::app::DrawCtx;
use crate::tui::strings as s;
use crate::tui::theme;
use crate::tui::widgets;

use super::state::WizardState;
use super::step::WStep;

/// How long a per-block `copied ✓` / save flash stays lit.
pub(super) const FLASH: Duration = Duration::from_secs(2);
/// The status-line flash (copy-all / save-path) lingers a touch longer.
pub(super) const STATUS_FLASH: Duration = Duration::from_secs(4);

impl WizardState {
    pub fn draw(&self, f: &mut Frame, area: Rect, _ctx: &DrawCtx) {
        match self.step {
            WStep::Welcome => self.draw_welcome(f, area),
            WStep::Check => self.draw_check(f, area),
            WStep::SelectDacs => self.draw_select(f, area),
            WStep::Review => self.draw_review(f, area),
            WStep::Test => self.draw_test(f, area),
            WStep::Done => self.draw_done(f, area),
        }

        // Overlays (Check override select, manual node input) on top.
        if let Some((_, popup)) = &self.check_editor {
            popup.draw(f, area);
        }
        if let Some(input) = &self.manual {
            let body = format!("{}\n\n> {}", s::WIZ_MANUAL_BODY, input.display());
            widgets::modal(f, area, s::WIZ_MANUAL_TITLE, &body, s::HELP_INPUT);
        }
    }

    pub(super) fn draw_welcome(&self, f: &mut Frame, area: Rect) {
        let mut lines = vec![
            Line::from(Span::styled(s::WIZ_WELCOME_TITLE, theme::accent_bold())),
            widgets::blank(),
        ];
        // Word-wrap the body, preserving intentional paragraph breaks (FB5).
        for l in s::WIZ_WELCOME_BODY.split('\n') {
            if l.trim().is_empty() {
                lines.push(widgets::blank());
            } else {
                for wl in widgets::wrap(l, area.width.max(1) as usize) {
                    lines.push(Line::from(wl));
                }
            }
        }
        lines.push(widgets::blank());
        lines.push(Line::from(widgets::help_spans(s::WIZ_WELCOME_CTA)));
        f.render_widget(Paragraph::new(lines), area);
    }
}
