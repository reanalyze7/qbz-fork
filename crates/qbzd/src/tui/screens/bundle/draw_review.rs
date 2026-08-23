use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::Frame;

use crate::tui::strings as s;
use crate::tui::widgets;

use super::bucket::{bucket, BucketKind};
use super::state::BundleState;

impl BundleState {
    pub(super) fn draw_review(&self, f: &mut Frame, area: Rect) {
        let Some(p) = &self.pending else { return };
        let mut lines: Vec<Line> = Vec::new();

        bucket(&mut lines, s::B_BUCKET_APPLIED, &p.plan.applied, BucketKind::Applied);
        lines.push(widgets::blank());
        bucket(&mut lines, s::B_BUCKET_ADAPTED, &p.plan.adapted, BucketKind::Adapted);
        if p.plan.device_pick.is_some() {
            lines.push(widgets::note_line("press p to pick a local device for the missing one"));
        }
        lines.push(widgets::blank());
        bucket(&mut lines, s::B_BUCKET_SKIPPED, &p.plan.skipped, BucketKind::Skipped);

        widgets::panel(f, area, s::BUNDLE_TITLE, lines, self.scroll);

        if self.device_picker.is_some() {
            self.device_picker.as_ref().unwrap().draw(f, area);
        } else if self.auth_confirm {
            widgets::modal(
                f,
                area,
                s::B_IMPORT_AUTH_TITLE,
                s::B_IMPORT_AUTH_BODY,
                s::B_IMPORT_AUTH_HINT,
            );
        }
    }
}
