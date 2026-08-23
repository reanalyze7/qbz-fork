// crates/qbzd/src/tui/screens/wizard/draw_check.rs — the Check step's render
// (health verdict, distro/init override rows, remediation commands).

use qbz_audio::{Distro, InitSystem, Sandbox};
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::tui::strings as s;
use crate::tui::theme;
use crate::tui::widgets;
use crate::tui::wizard_core;

use super::state::WizardState;

impl WizardState {
    pub(super) fn draw_check(&self, f: &mut Frame, area: Rect) {
        let distro = Distro::ALL.get(self.distro_index).copied().unwrap_or(Distro::Other);
        let init = InitSystem::ALL.get(self.init_index).copied().unwrap_or(InitSystem::Unknown);
        let width = area.width;
        let mut lines: Vec<Line> = Vec::new();

        // Health verdict (blind inside a sandbox — show reference commands only).
        if self.sandbox != Sandbox::None {
            lines.extend(widgets::wrapped_note(&s::wiz_sandbox_note(sandbox_name(self.sandbox)), width, theme::warn()));
        } else if let Some(h) = &self.health {
            if h.is_ready() {
                lines.push(Line::from(Span::styled(s::WIZ_HEALTH_READY, theme::ok())));
            } else {
                lines.push(Line::from(Span::styled(s::WIZ_HEALTH_ATTENTION, theme::warn())));
            }
        } else {
            lines.push(Line::from(Span::styled(s::WIZ_HEALTH_CHECKING, theme::dim())));
        }
        lines.push(widgets::blank());

        // The two overrides (focusable rows), sharing one control column.
        let ctrl_col = widgets::control_column(&[s::WIZ_DISTRO, s::WIZ_INIT], width);
        lines.extend(self.check_block(0, s::WIZ_DISTRO, distro.label(), ctrl_col, width));
        lines.extend(self.check_block(1, s::WIZ_INIT, init.label(), ctrl_col, width));
        lines.push(widgets::blank());

        // Remediation / reference commands.
        let rows = if self.sandbox != Sandbox::None {
            wizard_core::reference_commands(distro, init)
        } else if let Some(h) = &self.health {
            wizard_core::remediations(*h, distro, init)
        } else {
            Vec::new()
        };
        if rows.is_empty() && self.sandbox == Sandbox::None && self.health.is_some() {
            lines.extend(widgets::wrapped_note(s::WIZ_NO_REMEDIATION, width, theme::dim()));
        }
        for (caption, command) in &rows {
            // Captions are prose → wrap; commands are copy-paste → never wrap.
            for cl in widgets::wrap(caption, width.saturating_sub(4).max(1) as usize) {
                lines.push(Line::from(Span::styled(format!("  • {cl}"), Style::default())));
            }
            for cmd_line in command.lines() {
                lines.push(Line::from(Span::styled(format!("      {cmd_line}"), theme::dim())));
            }
        }
        f.render_widget(Paragraph::new(lines), area);
    }

    pub(super) fn check_block(&self, idx: usize, label: &'static str, value: &str, ctrl_col: u16, width: u16) -> Vec<Line<'static>> {
        let focused = self.check_focus == idx && self.check_editor.is_none();
        widgets::field_block(
            &widgets::Field {
                label,
                value: value.to_string(),
                widget: "[select]",
                focused,
                enabled: true,
                reason: None,
                description: None,
            },
            ctrl_col,
            width,
        )
    }
}

pub(super) fn sandbox_name(sb: Sandbox) -> &'static str {
    match sb {
        Sandbox::Flatpak => "Flatpak",
        Sandbox::Snap => "Snap",
        Sandbox::None => "",
    }
}
