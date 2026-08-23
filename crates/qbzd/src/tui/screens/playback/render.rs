use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::Frame;

use crate::tui::app::DrawCtx;
use crate::tui::strings as s;
use crate::tui::widgets;

use super::fields::{visible_fields, PField};
use super::model::{Editor, PlaybackState};

impl PlaybackState {
    // -------------------------- render --------------------------

    pub fn draw(&self, f: &mut Frame, area: Rect, _ctx: &DrawCtx) {
        let width = area.width.saturating_sub(2); // section inner width
        let fields = visible_fields(&self.staged);
        let focused_field = fields.get(self.focus).copied();
        let active = |members: &[PField]| {
            focused_field.map(|ff| members.contains(&ff)).unwrap_or(false)
        };
        // ONE control column for the whole screen (owner's "misma área de columna").
        let labels: Vec<&str> = fields.iter().map(|f| self.field_display(*f).0).collect();
        let ctrl_col = widgets::control_column(&labels, width);

        use PField::*;
        let mut secs: Vec<widgets::Section> = Vec::new();
        let mut anchor: Option<widgets::FocusAnchor> = None;

        let quality: &[PField] = &[Quality, Limit, MaxRate, AllowFallback, RetryFail];
        let (q_lines, q_a) = self.group_block(&fields, quality, focused_field, ctrl_col, width);
        if !q_lines.is_empty() {
            widgets::push_section(&mut secs, &mut anchor, s::PLAYBACK_GROUP_QUALITY, active(quality), q_lines, q_a);
        }

        let behavior: &[PField] = &[Continue, Gapless];
        let (b_lines, b_a) = self.group_block(&fields, behavior, focused_field, ctrl_col, width);
        if !b_lines.is_empty() {
            widgets::push_section(&mut secs, &mut anchor, s::PLAYBACK_GROUP_BEHAVIOR, active(behavior), b_lines, b_a);
        }

        let session: &[PField] = &[Restore, Resume];
        let (sess_lines, sess_a) = self.group_block(&fields, session, focused_field, ctrl_col, width);
        if !sess_lines.is_empty() {
            widgets::push_section(&mut secs, &mut anchor, s::PLAYBACK_GROUP_SESSION, active(session), sess_lines, sess_a);
        }

        let controls: &[PField] = &[Mpris];
        let (ctl_lines, ctl_a) = self.group_block(&fields, controls, focused_field, ctrl_col, width);
        if !ctl_lines.is_empty() {
            widgets::push_section(&mut secs, &mut anchor, s::PLAYBACK_GROUP_CONTROLS, active(controls), ctl_lines, ctl_a);
        }

        widgets::sections_scroll(f, area, &secs, anchor);

        match &self.editor {
            Some(Editor::Quality(p)) | Some(Editor::MaxRate(p)) | Some(Editor::Retry(p)) => {
                p.draw(f, area)
            }
            None => {}
        }
    }

    /// The field blocks of one group, in declared order (skipping hidden fields).
    /// Returns the flattened lines plus, when the focused field is in this group,
    /// its (first-line, height) inside the group for follow-focus scrolling.
    fn group_block(
        &self,
        fields: &[PField],
        members: &[PField],
        focused_field: Option<PField>,
        ctrl_col: u16,
        width: u16,
    ) -> (Vec<Line<'static>>, Option<(u16, u16)>) {
        let mut lines = Vec::new();
        let mut within = None;
        for gf in members {
            if let Some(pos) = fields.iter().position(|x| x == gf) {
                let start = lines.len() as u16;
                let block = self.field_block(*gf, pos, ctrl_col, width);
                if focused_field == Some(*gf) {
                    within = Some((start, block.len() as u16));
                }
                lines.extend(block);
            }
        }
        (lines, within)
    }
}
