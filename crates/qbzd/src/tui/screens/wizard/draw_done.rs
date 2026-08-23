// crates/qbzd/src/tui/screens/wizard/draw_done.rs — the Done step's render
// (summary + service-restart reminder).

use qbz_audio::InitSystem;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::tui::strings as s;
use crate::tui::theme;
use crate::tui::widgets;
use crate::tui::wizard_core;

use super::state::WizardState;

impl WizardState {
    pub(super) fn draw_done(&self, f: &mut Frame, area: Rect) {
        let mut lines = vec![
            Line::from(ratatui::text::Span::styled(s::WIZ_DONE_TITLE, theme::accent_bold())),
            widgets::blank(),
        ];
        let selected = self.configs.len();
        for wl in widgets::wrap(&s::wiz_done_summary(selected), area.width.max(1) as usize) {
            lines.push(Line::from(wl));
        }
        lines.push(widgets::blank());
        lines.extend(widgets::wrapped_note(s::WIZ_DONE_REMINDER, area.width, theme::warn()));
        // The init-aware "(re)start the audio services" command for this box.
        let init = InitSystem::ALL.get(self.init_index).copied().unwrap_or(InitSystem::Unknown);
        lines.push(widgets::blank());
        lines.push(widgets::note_line(s::WIZ_DONE_RESTART));
        for cmd_line in wizard_core::restart_cmd(init).lines() {
            lines.push(widgets::note_line(cmd_line));
        }
        lines.push(widgets::blank());
        lines.push(Line::from(widgets::help_spans(s::WIZ_DONE_CTA)));
        f.render_widget(Paragraph::new(lines), area);
    }
}
