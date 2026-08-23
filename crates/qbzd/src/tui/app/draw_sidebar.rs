// crates/qbzd/src/tui/app/draw_sidebar.rs — the persistent left navigation
// (section list, dirty marker, focus highlight, FB5 wide-tier summaries).

use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Paragraph};
use ratatui::Frame;

use crate::tui::strings as s;
use crate::tui::theme;

use super::messages::SCREENS;
use super::nav::{sidebar_dirty_marker, Focus};
use super::state::App;

impl App {
    /// Persistent left navigation: the sections by name. The active one gets
    /// `▸` + accent; a dirty active section gets a warn `*`. When the nav owns
    /// focus, the highlighted NAME row reverses (serial-safe) and the border
    /// accents. In the wide tier (FB5) the labels spell out (`Import / Export`)
    /// and each name carries a dim static summary line beneath it.
    pub(super) fn draw_sidebar(&self, f: &mut Frame, area: Rect, wide: bool) {
        let border = if self.focus == Focus::Nav {
            theme::accent()
        } else {
            theme::dim()
        };
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(border);
        let inner = block.inner(area);
        f.render_widget(block, area);

        let width = inner.width as usize;
        let active_dirty = self.active_is_dirty();
        let mut lines: Vec<Line> = Vec::new();
        for (i, screen) in SCREENS.iter().enumerate() {
            let is_active = *screen == self.active_section;
            let dirty = sidebar_dirty_marker(*screen, self.active_section, active_dirty);
            let highlighted = self.focus == Focus::Nav && i == self.nav_cursor;
            let label = if wide { s::SIDEBAR_LABELS_WIDE[i] } else { s::SIDEBAR_LABELS[i] };
            let marker = if is_active { "▸ " } else { "  " };

            if highlighted {
                // Full-width reverse bar (accent-reversed) — reads on monochrome
                // and serial. Padded to the inner width so the bar spans the row.
                let dirty_str = if dirty { " *" } else { "" };
                let core = format!("{marker}{label}{dirty_str}");
                let padded = format!("{core:<width$}");
                lines.push(Line::from(Span::styled(padded, theme::selection())));
            } else {
                let mut spans = vec![
                    Span::styled(
                        marker.to_string(),
                        if is_active { theme::accent() } else { Style::default() },
                    ),
                    Span::styled(
                        label.to_string(),
                        if is_active { theme::accent_bold() } else { Style::default() },
                    ),
                ];
                if dirty {
                    spans.push(Span::styled(" *".to_string(), theme::warn()));
                }
                lines.push(Line::from(spans));
            }
            // Wide tier: a dim static summary under each name (never live state).
            if wide {
                lines.push(Line::from(Span::styled(
                    format!("    {}", s::SIDEBAR_SUMMARIES[i]),
                    theme::dim(),
                )));
            }
        }
        f.render_widget(Paragraph::new(lines), inner);
    }
}
