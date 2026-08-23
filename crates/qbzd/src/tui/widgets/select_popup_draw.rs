use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Clear, Paragraph};
use ratatui::Frame;

use super::super::{strings, theme};
use super::field::focus_style;
use super::help::help_spans;
use super::layout::centered_rect;
use super::lines::note_line;
use super::overlay::titled_block;
use super::select_popup::SelectPopup;

impl SelectPopup {
    pub fn draw(&self, f: &mut Frame, area: Rect) {
        let vis = self.visible();
        let show_headers = self.filter.is_empty();
        let mut lines: Vec<Line> = Vec::new();
        let mut sel_line: u16 = 0;
        for i in &vis {
            if show_headers {
                if let Some(Some(h)) = self.headers.get(*i) {
                    lines.push(Line::from(Span::styled(h.clone(), theme::accent_bold())));
                }
            }
            let style = if *i == self.idx {
                sel_line = lines.len() as u16;
                focus_style()
            } else {
                Style::default()
            };
            lines.push(Line::from(Span::styled(
                format!("  {}", self.options[*i]),
                style,
            )));
        }
        if vis.is_empty() {
            lines.push(note_line("(no matches)"));
        }
        let hint = if self.filterable {
            format!(
                "{}   [{}]",
                strings::HELP_FILTER,
                if self.filter.is_empty() { "type to filter".into() } else { self.filter.clone() }
            )
        } else {
            strings::HELP_SELECT.to_string()
        };

        let height = (lines.len() as u16 + 4)
            .min(area.height.saturating_sub(2))
            .max(6);
        let width = self
            .options
            .iter()
            .map(|o| o.chars().count())
            .chain(std::iter::once(hint.chars().count()))
            .chain(std::iter::once(self.title.chars().count()))
            .max()
            .unwrap_or(20) as u16
            + 8;
        let rect = centered_rect(width.min(area.width.saturating_sub(2)).max(24), height, area);
        f.render_widget(Clear, rect);
        let block = titled_block(&self.title);
        let inner = block.inner(rect);
        f.render_widget(block, rect);

        let list_h = inner.height.saturating_sub(1);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(list_h), Constraint::Length(1)])
            .split(inner);
        // Scroll so the selected line (accounting for header rows) stays visible.
        let scroll = sel_line.saturating_sub(list_h.saturating_sub(1));
        f.render_widget(Paragraph::new(lines).scroll((scroll, 0)), chunks[0]);
        f.render_widget(Paragraph::new(Line::from(help_spans(&hint))), chunks[1]);
    }
}
