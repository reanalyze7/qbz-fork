use ratatui::layout::{Alignment, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Clear, Paragraph, Wrap};
use ratatui::Frame;

use super::super::theme;
use super::help::help_spans;
use super::layout::centered_rect;
use super::lines::blank;

// ============================ centered modal ============================

/// A centered, bordered modal with a title, wrapped body and a hint footer.
pub fn modal(f: &mut Frame, area: Rect, title: &str, body: &str, hint: &str) {
    let lines = body.lines().count().max(1) as u16;
    let height = lines + 4; // border(2) + body + spacer + hint
    let width = body
        .lines()
        .chain(std::iter::once(title))
        .chain(std::iter::once(hint))
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(20) as u16
        + 6;
    let rect = centered_rect(width.max(28), height.max(6), area);
    f.render_widget(Clear, rect);
    let block = titled_block(title);
    let inner = block.inner(rect);
    f.render_widget(block, rect);

    let mut text: Vec<Line> = body.lines().map(|l| Line::from(l.to_string())).collect();
    text.push(blank());
    text.push(Line::from(help_spans(hint)));
    f.render_widget(Paragraph::new(text).wrap(Wrap { trim: false }), inner);
}

/// A rounded, accent-bordered block with an accent-bold title — the shared frame
/// for modals, popups and panels.
pub(super) fn titled_block(title: &str) -> Block<'static> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(theme::accent())
        .title(Line::from(Span::styled(format!(" {title} "), theme::accent_bold())))
}

/// A centered scrollable panel (help overlay, import summary, result panel).
pub fn panel(f: &mut Frame, area: Rect, title: &str, lines: Vec<Line<'static>>, scroll: u16) {
    let rect = centered_rect(area.width.saturating_sub(6).max(40), area.height.saturating_sub(4).max(10), area);
    f.render_widget(Clear, rect);
    let block = titled_block(title);
    let inner = block.inner(rect);
    f.render_widget(block, rect);
    f.render_widget(
        Paragraph::new(lines).scroll((scroll, 0)).wrap(Wrap { trim: false }),
        inner,
    );
}

/// Spinner glyph for the given tick (§5.5 worker spinner).
pub fn spinner_frame(tick: u64) -> char {
    const FRAMES: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    FRAMES[(tick as usize) % FRAMES.len()]
}

/// Busy overlay: a small centered spinner + label (§5.5). The spinner glyph is
/// accent; the label plain.
pub fn busy_overlay(f: &mut Frame, area: Rect, label: &str, tick: u64) {
    // Layout parity with the pre-theme overlay: body = "<spinner> <label>"
    // (label + 2 chars) + 6 → total rect width = label + 8.
    let width = label.chars().count() as u16 + 8;
    let rect = centered_rect(width, 5, area);
    f.render_widget(Clear, rect);
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(theme::accent());
    let inner = block.inner(rect);
    f.render_widget(block, rect);
    let line = Line::from(vec![
        Span::styled(format!("{} ", spinner_frame(tick)), theme::accent()),
        Span::raw(label.to_string()),
    ]);
    f.render_widget(
        Paragraph::new(line).alignment(Alignment::Center),
        inner,
    );
}
