use std::net::IpAddr;

use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::Frame;

use crate::tui::app::DrawCtx;
use crate::tui::strings as s;
use crate::tui::theme;
use crate::tui::widgets;

use super::state::{NField, NetworkState, FIELDS};

impl NetworkState {
    // -------------------------- render --------------------------

    pub fn draw(&self, f: &mut Frame, area: Rect, _ctx: &DrawCtx) {
        let width = area.width.saturating_sub(2); // section inner width
        let labels: Vec<&str> = FIELDS
            .iter()
            .map(|f| match f {
                NField::Bind => s::N_BIND,
                NField::Port => s::N_PORT,
                NField::Token => s::N_TOKEN,
            })
            .collect();
        let ctrl_col = widgets::control_column(&labels, width);

        let mut lines: Vec<Line> = Vec::new();
        let mut within: Option<(u16, u16)> = None;
        for (i, field) in FIELDS.iter().enumerate() {
            let focused = i == self.focus && self.editor.is_none();
            let editing = self.editor.as_ref().map(|(nf, _)| *nf == *field).unwrap_or(false);
            let (label, value, widget) = match field {
                NField::Bind => (s::N_BIND, self.field_value(NField::Bind, editing), "[input]"),
                NField::Port => (s::N_PORT, self.field_value(NField::Port, editing), "[input]"),
                NField::Token => {
                    let v = if editing {
                        self.editor.as_ref().map(|(_, i)| i.display()).unwrap_or_default()
                    } else if self.staged.token.trim().is_empty() {
                        s::N_TOKEN_HINT.to_string()
                    } else {
                        widgets::mask(&self.staged.token)
                    };
                    (s::N_TOKEN, v, "[input]")
                }
            };
            let start = lines.len() as u16;
            let block = widgets::field_block(
                &widgets::Field { label, value, widget, focused, enabled: true, reason: None, description: None },
                ctrl_col,
                width,
            );
            if focused {
                within = Some((start, block.len() as u16));
            }
            lines.extend(block);
        }

        // Field-level validation notes (§4.2): the short errors stay one line; the
        // long exposure/preservation copy is now WORD-WRAPPED so it never clips.
        if self.staged.bind.parse::<IpAddr>().is_err() {
            lines.push(widgets::err_line(s::N_BAD_IP));
        } else if self.bind_is_lan() {
            lines.push(widgets::blank());
            lines.extend(widgets::wrapped_note(s::NETWORK_LAN_POSTURE, width, theme::dim()));
        }
        if self.port_invalid() {
            lines.push(widgets::err_line(s::N_BAD_PORT));
        }

        // Pre-save unknown-key warning (§3.5).
        if !self.unknown_keys.is_empty() {
            lines.push(widgets::blank());
            lines.extend(widgets::wrapped_note(s::N_DROP_UNKNOWN, width, theme::warn()));
            lines.extend(widgets::wrapped_note(&self.unknown_keys.join(", "), width, theme::warn()));
        }

        let mut secs: Vec<widgets::Section> = Vec::new();
        let mut anchor: Option<widgets::FocusAnchor> = None;
        widgets::push_section(&mut secs, &mut anchor, s::NETWORK_SECTION, true, lines, within);
        widgets::sections_scroll(f, area, &secs, anchor);

        if let Some((field, input)) = &self.editor {
            let title = match field {
                NField::Bind => s::N_BIND,
                NField::Port => s::N_PORT,
                NField::Token => s::N_TOKEN,
            };
            widgets::modal(f, area, title, &input.display(), s::HELP_INPUT);
        }
    }

    fn field_value(&self, field: NField, editing: bool) -> String {
        if editing {
            if let Some((_, input)) = &self.editor {
                return input.display();
            }
        }
        match field {
            NField::Bind => self.staged.bind.clone(),
            NField::Port => self.staged.port.clone(),
            NField::Token => self.staged.token.clone(),
        }
    }
}
