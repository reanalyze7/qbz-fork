use ratatui::style::Style;
use ratatui::text::{Line, Span};

use super::super::theme;
use super::layout::wrap;
use super::text_fit::{pad_to, truncate};

/// Focused-row emphasis: accent reversed (§1.2 — reverse is terminal-theme
/// agnostic, so the selection reads even on monochrome/serial).
pub fn focus_style() -> Style {
    theme::selection()
}

/// Mask a secret for display (token/paste rows never render plaintext).
pub fn mask(s: &str) -> String {
    "•".repeat(s.chars().count())
}

// ============================ field blocks (FB5) ============================

/// One field to render as a block. Row 1 is `label` + the CONTROL (value +
/// right-aligned widget marker) anchored at a screen-consistent column; rows
/// 2..n are the wrapped, dim DESCRIPTION under the label — the disabled `reason`
/// takes precedence over the static `description`.
pub struct Field<'a> {
    pub label: &'a str,
    pub value: String,
    /// `[select]`/`[toggle]`/`[input]`/`[slider]`, or `""` for a plain value.
    pub widget: &'a str,
    pub focused: bool,
    pub enabled: bool,
    /// Why the field is inert (only meaningful when `!enabled`).
    pub reason: Option<&'a str>,
    /// Static one-line help; wrapped under the label when present.
    pub description: Option<&'a str>,
}

/// Render a field as a block of lines (FB5). `ctrl_col` is the shared control
/// column; `width` is the section inner width. The value is truncated with `…`
/// (values never wrap — the control stays one line); the widget marker is
/// right-aligned so the marker column reads cleanly too. When `focused` the whole
/// control row is one accent-reverse bar (serial-safe, §1.2).
pub fn field_block(field: &Field, ctrl_col: u16, width: u16) -> Vec<Line<'static>> {
    let width = width.max(ctrl_col + 4) as usize;
    let ctrl = ctrl_col as usize;
    let widget_len = field.widget.chars().count();
    // Reserve the widget + a one-column gap on the right; the rest is the value.
    let reserved = if widget_len == 0 { 0 } else { widget_len + 1 };
    let value_space = width.saturating_sub(ctrl + reserved);
    let value = truncate(&field.value, value_space);
    let value_len = value.chars().count();

    let label_piece = pad_to(&format!("  {}", field.label), ctrl);
    let mid_pad = width.saturating_sub(ctrl + value_len + widget_len);
    let value_piece = format!("{value}{}", " ".repeat(mid_pad));
    let widget_piece = field.widget.to_string();

    let row1 = if field.focused {
        let mut sel = focus_style();
        if !field.enabled {
            sel = sel.patch(theme::dim());
        }
        Line::from(vec![
            Span::styled(label_piece, sel),
            Span::styled(value_piece, sel),
            Span::styled(widget_piece, sel),
        ])
    } else {
        let label_style = if field.enabled { Style::default() } else { theme::dim() };
        let value_style = if !field.enabled {
            theme::dim()
        } else if field.widget == "[toggle]" {
            toggle_tone(&field.value)
        } else {
            Style::default()
        };
        Line::from(vec![
            Span::styled(label_piece, label_style),
            Span::styled(value_piece, value_style),
            Span::styled(widget_piece, theme::dim()),
        ])
    };

    let mut out = vec![row1];
    // Disabled reason wins over the static description; both wrap under the label.
    let desc = if !field.enabled {
        field.reason.or(field.description)
    } else {
        field.description
    };
    if let Some(text) = desc {
        for wl in wrap(text, width.saturating_sub(4)) {
            out.push(Line::from(Span::styled(format!("    {wl}"), theme::dim())));
        }
    }
    out
}

/// ok for an on/enabled toggle, dim for off — the text ("on"/"off") carries the
/// meaning; the tint only reinforces it.
fn toggle_tone(value: &str) -> Style {
    if value == "off" {
        theme::dim()
    } else {
        theme::ok()
    }
}
