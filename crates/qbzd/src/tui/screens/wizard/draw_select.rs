// crates/qbzd/src/tui/screens/wizard/draw_select.rs — the Select-DACs step's
// render (candidate checklist + manual escape hatch).

use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::tui::strings as s;
use crate::tui::theme;
use crate::tui::widgets;
use crate::tui::wizard_core;

use super::draw::STATUS_FLASH;
use super::state::WizardState;

impl WizardState {
    pub(super) fn draw_select(&self, f: &mut Frame, area: Rect) {
        let mut lines: Vec<Line> = Vec::new();
        if self.detecting {
            lines.push(Line::from(Span::styled(s::WIZ_DETECTING, theme::dim())));
        } else if self.candidates.is_empty() {
            lines.extend(widgets::wrapped_note(s::WIZ_NO_DACS, area.width, theme::warn()));
        } else {
            lines.extend(widgets::wrapped_note(s::WIZ_SELECT_INTRO, area.width, theme::dim()));
            lines.push(widgets::blank());
            for (i, c) in self.candidates.iter().enumerate() {
                let mark = if c.checked { "[x]" } else { "[ ]" };
                let badge = if c.looks_like_dac { s::WIZ_DAC_BADGE } else { "" };
                let deflt = if c.is_default { s::WIZ_DEFAULT_BADGE } else { "" };
                let bus = if c.bus.is_empty() { String::new() } else { format!(" ({})", c.bus) };
                let text = format!("{mark} {}{bus}{badge}{deflt}", c.description);
                let focused = i == self.dac_focus;
                let style = if focused { theme::selection() } else { Style::default() };
                lines.push(Line::from(Span::styled(format!("  {text}"), style)));
                if !c.rates_label.is_empty() {
                    lines.push(widgets::note_line(&format!("supports {}", c.rates_label)));
                }
            }
        }
        // Manual escape hatch + accepted node.
        lines.push(widgets::blank());
        if let Some(m) = &self.manual_node {
            lines.push(widgets::note_line(&format!(
                "{} {m} ({})",
                s::WIZ_MANUAL_ACCEPTED,
                wizard_core::detect_dac_type(m)
            )));
        }
        lines.push(widgets::note_line(s::WIZ_MANUAL_HINT));
        if let Some((note, at)) = &self.gate_note {
            if at.elapsed() < STATUS_FLASH {
                lines.push(widgets::warn_line(note));
            }
        }
        f.render_widget(Paragraph::new(lines), area);
    }
}
