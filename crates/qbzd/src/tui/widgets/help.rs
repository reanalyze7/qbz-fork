use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use super::super::theme;

// ============================ help bar ============================

/// True only for tokens that are real key glyphs in the hint vocabulary: a
/// single character (`s`, `r`, `/`, `?`, `q`, `y`, `d`, `p`, …) or a named key.
/// Instructional words ("type") are NOT keys — their segment stays dim.
fn is_key_glyph(token: &str) -> bool {
    token.chars().count() == 1
        || matches!(
            token,
            "Esc" | "Enter" | "Tab" | "Shift-Tab" | "up/down" | "left/right" | "up" | "down"
        )
}

/// Split a `key desc · key desc` hint into accent-key / dim-description spans.
/// Segments are separated by ` · `; a segment's leading token is accent-tinted
/// only when it is a real key glyph — otherwise the whole segment is dim.
pub fn help_spans(text: &str) -> Vec<Span<'static>> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    for (i, seg) in text.split(" · ").enumerate() {
        if i > 0 {
            spans.push(Span::styled(" · ", theme::dim()));
        }
        match seg.split_once(' ') {
            Some((key, rest)) if is_key_glyph(key) => {
                spans.push(Span::styled(key.to_string(), theme::accent()));
                spans.push(Span::styled(format!(" {rest}"), theme::dim()));
            }
            None if is_key_glyph(seg) => {
                spans.push(Span::styled(seg.to_string(), theme::accent()));
            }
            _ => spans.push(Span::styled(seg.to_string(), theme::dim())),
        }
    }
    spans
}

/// The bottom help bar (one line): accent key glyphs, dim descriptions.
pub fn help_bar(f: &mut Frame, area: Rect, text: &str) {
    let mut spans = vec![Span::raw(" ")];
    spans.extend(help_spans(text));
    f.render_widget(Paragraph::new(Line::from(spans)), area);
}
