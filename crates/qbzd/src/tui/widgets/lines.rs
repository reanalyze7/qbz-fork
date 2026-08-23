use ratatui::style::Style;
use ratatui::text::{Line, Span};

use super::super::theme;
use super::field::focus_style;
use super::layout::wrap;

/// A plain, non-interactive info/action line (e.g. Account's status row or an
/// action item). `focused` gives it the accent bar.
pub fn action_line(text: &str, focused: bool, enabled: bool) -> Line<'static> {
    let style = if focused {
        focus_style()
    } else if !enabled {
        theme::dim()
    } else {
        Style::default()
    };
    Line::from(Span::styled(format!("  {text}"), style))
}

pub fn blank() -> Line<'static> {
    Line::from("")
}

/// A dim, wrapped note under a field (previews, hints).
pub fn note_line(text: &str) -> Line<'static> {
    Line::from(Span::styled(format!("    {text}"), theme::dim()))
}

/// A warn-tinted note (LAN exposure, DSD/auth safety, unknown-key preservation).
/// The copy already reads as a warning; the tint is a second channel, not the
/// only one.
pub fn warn_line(text: &str) -> Line<'static> {
    Line::from(Span::styled(format!("    {text}"), theme::warn()))
}

/// An error-tinted note (rejected bind/port). Same rule: the text stands alone.
pub fn err_line(text: &str) -> Line<'static> {
    Line::from(Span::styled(format!("    {text}"), theme::err()))
}

/// A multi-line note WORD-WRAPPED to `width` (FB5) — the long LAN/auth/export
/// copy no longer clips at the frame edge. `style` picks the tone (dim note,
/// warn, err). The 4-space indent matches `note_line`.
pub fn wrapped_note(text: &str, width: u16, style: Style) -> Vec<Line<'static>> {
    let inner = (width as usize).saturating_sub(4).max(1);
    wrap(text, inner)
        .into_iter()
        .map(|l| Line::from(Span::styled(format!("    {l}"), style)))
        .collect()
}
