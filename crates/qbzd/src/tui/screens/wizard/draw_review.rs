// crates/qbzd/src/tui/screens/wizard/draw_review.rs — the Review step's
// render (one bordered block per DAC) + its line-counting helpers, which
// `keys_review.rs` also uses to keep scroll following the focused block.

use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::tui::strings as s;
use crate::tui::theme;
use crate::tui::widgets;
use crate::tui::wizard_core::DacConfigData;

use super::draw::{FLASH, STATUS_FLASH};
use super::state::WizardState;
use super::state_types::ConfigBlock;

impl WizardState {
    pub(super) fn draw_review(&self, f: &mut Frame, area: Rect) {
        if self.configs.is_empty() {
            let l = vec![Line::from(Span::styled(s::WIZ_GENERATING, theme::dim()))];
            f.render_widget(Paragraph::new(l), area);
            return;
        }

        // Reserve the bottom row for the status flash + a fixed footer line.
        let body_h = area.height.saturating_sub(1);
        let body = Rect { height: body_h, ..area };

        let mut lines: Vec<Line> = Vec::new();
        // A backup reminder above the blocks (dim).
        lines.push(widgets::note_line(s::WIZ_BACKUP_HINT));
        for (i, block) in self.configs.iter().enumerate() {
            let focused = i == self.review_focus;
            append_block_lines(&mut lines, block, focused);
        }
        f.render_widget(Paragraph::new(lines).scroll((self.review_scroll, 0)), body);

        // Footer: transient status flash (copy/save result) else the safety note.
        let footer = Rect { y: area.y + area.height.saturating_sub(1), height: 1, ..area };
        let footer_line = match &self.status_flash {
            Some((msg, at)) if at.elapsed() < STATUS_FLASH => {
                Line::from(Span::styled(format!(" {msg}"), theme::ok()))
            }
            _ => Line::from(Span::styled(format!(" {}", s::WIZ_REVIEW_FOOTER), theme::dim())),
        };
        f.render_widget(Paragraph::new(footer_line), footer);
    }
}

/// Number of rendered lines one Review block occupies (header + paths + config
/// body + separator) — used to bring the focused block to the viewport top.
pub(super) fn block_line_count(data: &DacConfigData) -> u16 {
    // header(1) + paths(3) + config body + blank(1).
    let body = data.full_block().lines().count() as u16;
    1 + 3 + body + 1
}

/// One Review block, left-ruled with an accent (focused) / dim rail so it reads
/// as a bordered box while long config lines can run past the frame edge (the
/// FULL verbatim text is what `c`/`w` copy, not the clipped preview).
fn append_block_lines(lines: &mut Vec<Line<'static>>, block: &ConfigBlock, focused: bool) {
    let rail_style = if focused { theme::accent() } else { theme::dim() };
    let flashing = block
        .flash
        .as_ref()
        .map(|(_, at)| at.elapsed() < FLASH)
        .unwrap_or(false);

    // Header: rail + DAC name (+ the copied ✓ flash).
    let mut header = vec![
        Span::styled("│ ".to_string(), rail_style),
        Span::styled(
            block.data.name.clone(),
            if focused { theme::accent_bold() } else { Style::default() },
        ),
    ];
    if flashing {
        if let Some((tier, _)) = &block.flash {
            header.push(Span::styled(format!("   {}", tier.short_label()), theme::ok()));
        }
    }
    lines.push(Line::from(header));

    // Target files (dim).
    for path in block.data.target_paths() {
        lines.push(Line::from(vec![
            Span::styled("│ ".to_string(), rail_style),
            Span::styled(format!("→ {path}"), theme::dim()),
        ]));
    }
    // Config body.
    for l in block.data.full_block().lines() {
        lines.push(Line::from(vec![
            Span::styled("│ ".to_string(), rail_style),
            Span::styled(l.to_string(), Style::default()),
        ]));
    }
    lines.push(widgets::blank());
}
