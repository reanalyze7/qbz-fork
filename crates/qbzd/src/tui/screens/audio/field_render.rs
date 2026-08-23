use crate::tui::strings as s;
use crate::tui::widgets;

use super::fields::{row_state, AField};
use super::labels::{alsa_plugin_label, backend_label, dsd_label, short_device};
use super::state::AudioState;

impl AudioState {
    pub(super) fn field_block(&self, field: AField, focus_pos: usize, ctrl_col: u16, width: u16) -> Vec<ratatui::text::Line<'static>> {
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
            description: None,
        };
        widgets::field_block(&f, ctrl_col, width)
    }

    pub(super) fn field_display(&self, field: AField) -> (&'static str, String, &'static str) {
        let a = &self.staged;
        let on_off = |b: bool| if b { "on".to_string() } else { "off".to_string() };
        match field {
            AField::Backend => (s::A_BACKEND, backend_label(a.backend), "[select]"),
            AField::Device => {
                let dev = if self.scanning {
                    s::AUDIO_SCANNING.to_string()
                } else {
                    self.device_label()
                };
                (s::A_DEVICE, dev, "[select]")
            }
            AField::AlsaPlugin => (s::A_ALSA_PLUGIN, alsa_plugin_label(a.alsa_plugin).to_string(), "[select]"),
            AField::HwVolume => (s::A_HW_VOLUME, on_off(a.alsa_hardware_volume), "[toggle]"),
            AField::Dsd => (s::A_DSD, dsd_label(&a.dsd_mode).to_string(), "[select]"),
            AField::Exclusive => (s::A_EXCLUSIVE, on_off(a.exclusive_mode), "[toggle]"),
            AField::Reserve => (s::A_RESERVE, on_off(a.reserve_dac), "[toggle]"),
            AField::Passthrough => (s::A_PASSTHROUGH, on_off(a.dac_passthrough), "[toggle]"),
            AField::ForceBp => (s::A_FORCE_BP, on_off(a.pw_force_bitperfect), "[toggle]"),
            AField::LockOutput => (s::A_LOCK_OUTPUT, on_off(a.skip_sink_switch), "[toggle]"),
            AField::StreamUncached => (s::A_STREAM_UNCACHED, on_off(a.stream_first_track), "[toggle]"),
            AField::Buffer => (s::A_BUFFER, format!("{} s", a.stream_buffer_seconds), "[slider]"),
            AField::StreamingOnly => (s::A_STREAMING_ONLY, on_off(a.streaming_only), "[toggle]"),
        }
    }

    fn device_label(&self) -> String {
        match &self.staged.output_device {
            None => "System default".to_string(),
            Some(id) => self
                .devices
                .iter()
                .find(|d| &d.id == id)
                .map(|d| if d.bp { format!("{} {}", d.label, s::BP_BADGE) } else { d.label.clone() })
                .unwrap_or_else(|| short_device(id)),
        }
    }
}
