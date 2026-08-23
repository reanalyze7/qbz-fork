// crates/qbzd/src/tui/screens/wizard/draw_test.rs — the Test step's render
// (requested vs negotiated rate, reference seed tracks).

use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::tui::strings as s;
use crate::tui::theme;
use crate::tui::widgets;
use crate::tui::wizard_core;

use super::state::WizardState;

impl WizardState {
    pub(super) fn draw_test(&self, f: &mut Frame, area: Rect) {
        let mut lines: Vec<Line> = Vec::new();
        lines.extend(widgets::wrapped_note(s::WIZ_TEST_INTRO, area.width, theme::dim()));
        lines.push(widgets::blank());

        if let Some(note) = &self.test_note {
            lines.push(widgets::warn_line(note));
        }
        if self.tested {
            // Requested (what QBZ asked the daemon for) vs negotiated (the DAC's
            // real hardware clock) — the bit-perfect proof.
            let req = match self.test_requested {
                Some((rate, bits)) if rate > 0 => {
                    format!("QBZ requesting {} · {}-bit", wizard_core::khz(rate), bits)
                }
                _ => s::WIZ_TEST_NOTHING.to_string(),
            };
            lines.push(Line::from(Span::styled(format!("  {req}"), Style::default())));
            match &self.test_negotiated {
                Some(n) => {
                    let matched = self
                        .test_requested
                        .map(|(r, _)| r > 0 && n.sample_rate == r)
                        .unwrap_or(false);
                    let style = if matched { theme::ok() } else { theme::warn() };
                    lines.push(Line::from(Span::styled(
                        format!("  {}", wizard_core::negotiated_label(n)),
                        style,
                    )));
                    if matched {
                        lines.push(widgets::note_line(s::WIZ_TEST_MATCHED));
                    }
                    // Label a known reference track if the rate/depth lines up.
                    if let Some((rate, bits)) = self.test_requested {
                        if let Some(seed) = wizard_core::seed_for_rate_depth(rate, bits) {
                            lines.push(widgets::note_line(&format!(
                                "{} {} — {}",
                                s::WIZ_TEST_REFERENCE,
                                seed.artist,
                                seed.title
                            )));
                        }
                    }
                }
                None => lines.push(widgets::note_line(s::WIZ_TEST_WAITING)),
            }
        }

        lines.push(widgets::blank());
        // Reference seed tracks the operator can cast to verify each rate.
        lines.push(widgets::note_line(s::WIZ_TEST_SEEDS_HEADER));
        for seed in wizard_core::TEST_SEEDS.iter() {
            lines.push(widgets::note_line(&format!(
                "{}-bit/{} — {} — {} (Qobuz id {})",
                seed.depth,
                wizard_core::khz(seed.rate as u32),
                seed.artist,
                seed.title,
                seed.id_hint
            )));
        }
        f.render_widget(Paragraph::new(lines), area);
    }
}
