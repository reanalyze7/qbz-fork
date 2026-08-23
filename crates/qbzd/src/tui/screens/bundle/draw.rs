use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::Frame;

use crate::tui::app::DrawCtx;
use crate::tui::strings as s;
use crate::tui::theme;
use crate::tui::widgets;

use super::state::{BField, BundleState, Editor, FIELDS};

impl BundleState {
    // -------------------------- render --------------------------

    pub fn draw(&self, f: &mut Frame, area: Rect, _ctx: &DrawCtx) {
        if self.pending.is_some() {
            self.draw_review(f, area);
            return;
        }

        let width = area.width.saturating_sub(2); // section inner width
        let cur = FIELDS[self.focus];
        let ed = self.editor.is_none();
        // ONE control column across BOTH boxes (consistent for the whole screen).
        let ctrl_col = widgets::control_column(
            &[s::B_IMPORT_PATH, s::B_EXPORT_DEST, s::B_EXPORT_INCLUDE_AUTH],
            width,
        );

        let mut secs: Vec<widgets::Section> = Vec::new();
        let mut anchor: Option<widgets::FocusAnchor> = None;

        // IMPORT box.
        let path_val = if self.import_path.is_empty() {
            s::B_IMPORT_PATH_HINT.to_string()
        } else {
            self.import_path.clone()
        };
        let mut import_lines: Vec<Line> = Vec::new();
        let mut import_within: Option<(u16, u16)> = None;
        self.push_field(&mut import_lines, &mut import_within, cur, ed, BField::ImportPath, s::B_IMPORT_PATH, path_val, "[input]", ctrl_col, width);
        self.push_action(&mut import_lines, &mut import_within, cur, ed, BField::Review, s::B_IMPORT_ACTION);
        let import_active = matches!(cur, BField::ImportPath | BField::Review);
        widgets::push_section(&mut secs, &mut anchor, s::BUNDLE_IMPORT_HEADER, import_active, import_lines, import_within);

        // EXPORT box.
        let mut export_lines: Vec<Line> = Vec::new();
        let mut export_within: Option<(u16, u16)> = None;
        self.push_field(&mut export_lines, &mut export_within, cur, ed, BField::ExportDest, s::B_EXPORT_DEST, self.export_dest.clone(), "[input]", ctrl_col, width);
        self.push_field(&mut export_lines, &mut export_within, cur, ed, BField::IncludeAuth, s::B_EXPORT_INCLUDE_AUTH, if self.include_auth { "on" } else { "off" }.to_string(), "[toggle]", ctrl_col, width);
        if self.include_auth {
            export_lines.extend(widgets::wrapped_note(s::B_EXPORT_AUTH_WARNING, width, theme::warn()));
        }
        self.push_action(&mut export_lines, &mut export_within, cur, ed, BField::Export, s::B_EXPORT_ACTION);
        if self.has_desktop {
            export_lines.push(widgets::blank());
            export_lines.extend(widgets::wrapped_note(s::B_DESKTOP_HINT, width, theme::dim()));
        }
        let export_active = matches!(cur, BField::ExportDest | BField::IncludeAuth | BField::Export);
        widgets::push_section(&mut secs, &mut anchor, s::BUNDLE_EXPORT_HEADER, export_active, export_lines, export_within);

        widgets::sections_scroll(f, area, &secs, anchor);

        match &self.editor {
            Some(Editor::ImportPath(input)) => {
                widgets::modal(f, area, s::B_IMPORT_PATH, &input.display(), s::HELP_INPUT)
            }
            Some(Editor::ExportDest(input)) => {
                widgets::modal(f, area, s::B_EXPORT_DEST, &input.display(), s::HELP_INPUT)
            }
            None => {}
        }
    }

    /// Append a field block to `lines`, recording the focus anchor within the box.
    #[allow(clippy::too_many_arguments)]
    fn push_field(
        &self,
        lines: &mut Vec<Line<'static>>,
        within: &mut Option<(u16, u16)>,
        cur: BField,
        ed: bool,
        field: BField,
        label: &'static str,
        value: String,
        widget: &'static str,
        ctrl_col: u16,
        width: u16,
    ) {
        let focused = cur == field && ed;
        let start = lines.len() as u16;
        let block = widgets::field_block(
            &widgets::Field { label, value, widget, focused, enabled: true, reason: None, description: None },
            ctrl_col,
            width,
        );
        if focused {
            *within = Some((start, block.len() as u16));
        }
        lines.extend(block);
    }

    /// Append a single-line action row, recording the focus anchor within the box.
    fn push_action(
        &self,
        lines: &mut Vec<Line<'static>>,
        within: &mut Option<(u16, u16)>,
        cur: BField,
        ed: bool,
        field: BField,
        label: &str,
    ) {
        let focused = cur == field && ed;
        if focused {
            *within = Some((lines.len() as u16, 1));
        }
        lines.push(widgets::action_line(&format!("> {label}"), focused, true));
    }
}
