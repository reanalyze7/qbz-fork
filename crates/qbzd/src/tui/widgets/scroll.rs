use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Paragraph, Widget};
use ratatui::Frame;

use super::super::theme;
use super::layout::follow_scroll;
use super::section::{section_style, sections, sections_height, FocusAnchor, Section};

/// Stack section boxes like [`sections`], but follow-focus SCROLL when the
/// content is taller than `area` (FB5). When everything fits, it defers to
/// `sections` verbatim (top-aligned, no indicator). When it overflows, the boxes
/// render into a virtual buffer of the full height, the window that keeps the
/// focused block visible is blitted into `area`, and dim `▲`/`▼` indicators mark
/// hidden content above/below.
pub fn sections_scroll(f: &mut Frame, area: Rect, secs: &[Section], focus: Option<FocusAnchor>) {
    if secs.is_empty() {
        return;
    }
    let total = sections_height(secs);
    if total <= area.height {
        sections(f, area, secs);
        return;
    }

    let scroll = match &focus {
        Some(a) => {
            let mut y = 0u16;
            for s in secs.iter().take(a.section) {
                y = y.saturating_add(s.lines.len() as u16 + 2);
            }
            let focus_top = y + 1 + a.inner_line; // +1 for the box top border
            follow_scroll(focus_top, a.height, area.height, total)
        }
        None => 0,
    };

    // Render the stacked boxes into a full-height off-screen buffer.
    let mut buf = Buffer::empty(Rect { x: 0, y: 0, width: area.width, height: total });
    let mut y = 0u16;
    for sec in secs {
        let h = sec.lines.len() as u16 + 2;
        let rect = Rect { x: 0, y, width: area.width, height: h };
        let (border_style, title_style) = section_style(sec.active);
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(border_style)
            .title(Line::from(Span::styled(format!(" {} ", sec.title), title_style)));
        let inner = block.inner(rect);
        Widget::render(block, rect, &mut buf);
        Widget::render(Paragraph::new(sec.lines.clone()), inner, &mut buf);
        y = y.saturating_add(h);
    }

    // Blit the visible window into the frame.
    let fb = f.buffer_mut();
    for row in 0..area.height {
        let sy = scroll + row;
        if sy >= total {
            break;
        }
        for col in 0..area.width {
            if let Some(src) = buf.cell((col, sy)) {
                let cell = src.clone();
                if let Some(dst) = fb.cell_mut((area.x + col, area.y + row)) {
                    *dst = cell;
                }
            }
        }
    }

    // Dim scroll indicators at the right edge (content hidden above / below).
    let right = area.x + area.width.saturating_sub(1);
    if scroll > 0 {
        if let Some(c) = fb.cell_mut((right, area.y)) {
            c.set_symbol("▲");
            c.set_style(theme::dim());
        }
    }
    if scroll + area.height < total {
        if let Some(c) = fb.cell_mut((right, area.y + area.height.saturating_sub(1))) {
            c.set_symbol("▼");
            c.set_style(theme::dim());
        }
    }
}
