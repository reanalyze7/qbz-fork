use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::Frame;

use crate::tui::app::DrawCtx;
use crate::tui::strings as s;
use crate::tui::widgets;

use super::state::AccountState;

impl AccountState {
    // -------------------------- render --------------------------

    pub fn draw(&self, f: &mut Frame, area: Rect, _ctx: &DrawCtx) {
        let width = area.width.saturating_sub(2); // section inner width
        let ctrl_col = widgets::control_column(&[s::ACCOUNT_STATUS], width);
        let mut lines: Vec<Line> = Vec::new();

        // Status row — never fabricates a name (§3.1).
        let status = if self.auth.logged_in {
            match (&self.auth.email, &self.auth.plan) {
                (Some(e), Some(p)) => s::account_logged_in_plan(e, p),
                (Some(e), None) => s::account_logged_in(e),
                _ => "logged in".to_string(),
            }
        } else if self.auth.cred_file_present {
            s::ACCOUNT_CRED_PRESENT.to_string()
        } else {
            s::ACCOUNT_NOT_LOGGED_IN.to_string()
        };
        lines.extend(widgets::field_block(
            &widgets::Field {
                label: s::ACCOUNT_STATUS,
                value: status,
                widget: "",
                focused: false,
                enabled: true,
                reason: None,
                description: None,
            },
            ctrl_col,
            width,
        ));
        lines.push(widgets::blank());

        let mut anchor: Option<widgets::FocusAnchor> = None;
        for (i, action) in self.actions().iter().enumerate() {
            let focused = i == self.focus && !self.is_editing();
            if focused {
                anchor = Some(widgets::FocusAnchor { section: 0, inner_line: lines.len() as u16, height: 1 });
            }
            lines.push(widgets::action_line(&format!("> {action}"), focused, true));
        }

        let secs = [widgets::Section::new(s::ACCOUNT_SECTION, true, lines)];
        widgets::sections_scroll(f, area, &secs, anchor);

        if let Some(input) = &self.token_input {
            widgets::modal(f, area, s::ACCOUNT_PASTE_TOKEN, &input.display(), s::HELP_INPUT);
        } else if self.confirm_logout {
            widgets::modal(
                f,
                area,
                s::ACCOUNT_LOGOUT_CONFIRM_TITLE,
                s::ACCOUNT_LOGOUT_CONFIRM_BODY,
                s::CONFIRM_YN,
            );
        }
    }
}
