use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Paragraph};
use ratatui::Frame;

use super::super::theme;

// ============================ section boxes ============================

/// One titled, rounded section box's worth of content. `active` (the group that
/// owns the focused field) borders in accent; the rest border dim.
pub struct Section {
    pub title: String,
    pub active: bool,
    pub lines: Vec<Line<'static>>,
}

impl Section {
    pub fn new(title: impl Into<String>, active: bool, lines: Vec<Line<'static>>) -> Self {
        Self { title: title.into(), active, lines }
    }
}

/// Stack titled, rounded section boxes top-to-bottom in `area`. Each box is sized
/// to its content (+2 for the border); a trailing filler keeps them compact at
/// the top rather than stretching. The active box borders + titles in accent.
pub fn sections(f: &mut Frame, area: Rect, secs: &[Section]) {
    if secs.is_empty() {
        return;
    }
    let mut constraints: Vec<Constraint> = secs
        .iter()
        .map(|s| Constraint::Length(s.lines.len() as u16 + 2))
        .collect();
    constraints.push(Constraint::Min(0));
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);
    for (i, sec) in secs.iter().enumerate() {
        let (border_style, title_style) = section_style(sec.active);
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(border_style)
            .title(Line::from(Span::styled(format!(" {} ", sec.title), title_style)));
        let inner = block.inner(chunks[i]);
        f.render_widget(block, chunks[i]);
        f.render_widget(Paragraph::new(sec.lines.clone()), inner);
    }
}

pub(super) fn section_style(active: bool) -> (ratatui::style::Style, ratatui::style::Style) {
    if active {
        (theme::accent(), theme::accent_bold())
    } else {
        (theme::dim(), theme::dim())
    }
}

// ============================ follow-focus scroll (FB5) ============================

/// The virtual-line span of the focused field block inside a stacked-sections
/// render, so the viewport can scroll to keep it fully visible.
pub struct FocusAnchor {
    /// Index into the `secs` slice of the section that owns the focused block.
    pub section: usize,
    /// Line index of the block's first row WITHIN that section's `lines`.
    pub inner_line: u16,
    /// Rows the focused block occupies (control row + wrapped description).
    pub height: u16,
}

/// Push a section and, if `within` names the focused block's (line, height)
/// inside it, record the screen-wide [`FocusAnchor`] (FB5). Keeps every screen's
/// section assembly a one-liner instead of hand-tracking section indices.
pub fn push_section(
    secs: &mut Vec<Section>,
    anchor: &mut Option<FocusAnchor>,
    title: impl Into<String>,
    active: bool,
    lines: Vec<Line<'static>>,
    within: Option<(u16, u16)>,
) {
    if let Some((inner_line, height)) = within {
        *anchor = Some(FocusAnchor { section: secs.len(), inner_line, height });
    }
    secs.push(Section::new(title, active, lines));
}

/// Total rows the stacked section boxes need (each box = its lines + 2 borders).
pub(super) fn sections_height(secs: &[Section]) -> u16 {
    secs.iter().map(|s| s.lines.len() as u16 + 2).sum()
}
