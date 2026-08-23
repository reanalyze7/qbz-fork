use qbz_audio::AudioBackendType;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::Frame;

use crate::tui::app::DrawCtx;
use crate::tui::strings as s;
use crate::tui::theme;
use crate::tui::widgets;

use super::fields::{visible_fields, AField};
use super::state::{AudioState, Editor};

impl AudioState {
    // -------------------------- render --------------------------

    pub fn draw(&self, f: &mut Frame, area: Rect, _ctx: &DrawCtx) {
        let width = area.width.saturating_sub(2); // section inner width
        let fields = visible_fields(&self.staged);
        let focused_field = fields.get(self.focus).copied();
        let active = |members: &[AField]| {
            focused_field.map(|ff| members.contains(&ff)).unwrap_or(false)
        };
        // ONE control column for the whole screen (owner's "misma área de columna").
        let labels: Vec<&str> = fields.iter().map(|f| self.field_display(*f).0).collect();
        let ctrl_col = widgets::control_column(&labels, width);

        use AField::*;
        let mut secs: Vec<widgets::Section> = Vec::new();
        let mut anchor: Option<widgets::FocusAnchor> = None;

        let out_members: &[AField] = &[Backend, Device, AlsaPlugin, HwVolume, Dsd];
        let (mut out_lines, out_a) = self.group_block(&fields, out_members, focused_field, ctrl_col, width);
        if self.staged.backend == AudioBackendType::Jack {
            out_lines.extend(widgets::wrapped_note(s::JACK_WARNING, width, theme::warn()));
        }
        if !out_lines.is_empty() {
            widgets::push_section(&mut secs, &mut anchor, s::AUDIO_GROUP_OUTPUT, active(out_members), out_lines, out_a);
        }

        let bp_members: &[AField] = &[Exclusive, Reserve, Passthrough, ForceBp, LockOutput];
        let (bp_lines, bp_a) = self.group_block(&fields, bp_members, focused_field, ctrl_col, width);
        if !bp_lines.is_empty() {
            widgets::push_section(&mut secs, &mut anchor, s::AUDIO_GROUP_BITPERFECT, active(bp_members), bp_lines, bp_a);
        }

        let tr_members: &[AField] = &[StreamUncached, Buffer, StreamingOnly];
        let (tr_lines, tr_a) = self.group_block(&fields, tr_members, focused_field, ctrl_col, width);
        if !tr_lines.is_empty() {
            widgets::push_section(&mut secs, &mut anchor, s::AUDIO_GROUP_TRANSPORT, active(tr_members), tr_lines, tr_a);
        }

        widgets::sections_scroll(f, area, &secs, anchor);

        // Overlays.
        match &self.editor {
            Some(Editor::Backend(p))
            | Some(Editor::AlsaPlugin(p))
            | Some(Editor::Dsd(p)) => p.draw(f, area),
            Some(Editor::Device(p)) => {
                if self.scanning {
                    widgets::busy_overlay(f, area, s::AUDIO_SCANNING, 0);
                } else if self.devices.len() <= 1 {
                    // Only the synthetic "System default" — the §5.1 hint panel.
                    widgets::modal(f, area, s::DEVICE_PICKER_TITLE, s::NO_DEVICES, s::HELP_SELECT);
                } else {
                    p.draw(f, area);
                }
            }
            Some(Editor::DsdConfirm { .. }) => {
                widgets::modal(f, area, s::DSD_GUARD_TITLE, s::DSD_GUARD_BODY, s::DSD_GUARD_HINT);
            }
            None => {}
        }
    }

    /// The field blocks of one group, in declared order (skipping hidden fields).
    /// Returns the flattened lines plus, when the focused field is in this group,
    /// its (first-line, height) inside the group for follow-focus scrolling.
    fn group_block(
        &self,
        fields: &[AField],
        members: &[AField],
        focused_field: Option<AField>,
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
