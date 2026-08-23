use ratatui::text::Line;

use crate::tui::strings as s;
use crate::tui::widgets;

use super::fields::{row_state, PField};
use super::labels::{autoplay_label, max_rate_label, quality_label, retry_label};
use super::model::PlaybackState;

impl PlaybackState {
    pub(super) fn field_block(&self, field: PField, focus_pos: usize, ctrl_col: u16, width: u16) -> Vec<Line<'static>> {
        let (_, enabled, reason) = row_state(field, &self.staged);
        let focused = focus_pos == self.focus && self.editor.is_none();
        let (label, value, widget) = self.field_display(field);
        let f = widgets::Field {
            label,
            value,
            widget,
            focused,
            enabled,
            reason,
            description: field_description(field),
        };
        widgets::field_block(&f, ctrl_col, width)
    }

    pub(super) fn field_display(&self, field: PField) -> (&'static str, String, &'static str) {
        let a = &self.staged;
        let on_off = |b: bool| if b { "on".to_string() } else { "off".to_string() };
        match field {
            PField::Quality => (s::P_QUALITY, quality_label(&a.quality).to_string(), "[select]"),
            PField::Limit => (s::P_LIMIT_DEVICE, on_off(a.limit_to_device), "[toggle]"),
            PField::MaxRate => (s::P_MAX_RATE, max_rate_label(a.max_sample_rate).to_string(), "[select]"),
            PField::AllowFallback => (s::P_ALLOW_FALLBACK, on_off(a.allow_fallback), "[toggle]"),
            PField::RetryFail => (s::P_RETRY_FAIL, retry_label(&a.fallback_behavior).to_string(), "[select]"),
            PField::Continue => (s::P_CONTINUE, autoplay_label(&a.autoplay).to_string(), "[toggle]"),
            PField::Gapless => (s::P_GAPLESS, on_off(a.gapless), "[toggle]"),
            PField::Restore => (s::P_RESTORE, on_off(a.restore_session), "[toggle]"),
            PField::Resume => (s::P_RESUME_POS, on_off(a.resume_position), "[toggle]"),
            PField::Mpris => (s::P_MPRIS, on_off(a.mpris), "[toggle]"),
        }
    }
}

/// Static one-line help wrapped under a field's label. Only Mpris carries one
/// (it needs a restart to apply, unlike the live-ish other toggles).
fn field_description(field: PField) -> Option<&'static str> {
    match field {
        PField::Mpris => Some(s::P_MPRIS_DESC),
        _ => None,
    }
}
