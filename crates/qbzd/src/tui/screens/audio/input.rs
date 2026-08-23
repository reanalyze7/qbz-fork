use ratatui::crossterm::event::{KeyCode, KeyEvent};

use crate::tui::app::ScreenAction;

use super::cascades::cascade_on_toggle;
use super::fields::{row_state, visible_fields, AField};
use super::state::AudioState;

impl AudioState {
    // -------------------------- input --------------------------

    pub fn handle_key(&mut self, key: KeyEvent) -> ScreenAction {
        if self.editor.is_some() {
            return self.handle_editor_key(key);
        }
        let fields = visible_fields(&self.staged);
        if self.focus >= fields.len() {
            self.focus = fields.len().saturating_sub(1);
        }
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.focus == 0 {
                    self.focus = fields.len().saturating_sub(1);
                } else {
                    self.focus -= 1;
                }
                ScreenAction::Consumed
            }
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => {
                if !fields.is_empty() {
                    self.focus = (self.focus + 1) % fields.len();
                }
                ScreenAction::Consumed
            }
            KeyCode::BackTab => {
                if self.focus == 0 {
                    self.focus = fields.len().saturating_sub(1);
                } else {
                    self.focus -= 1;
                }
                ScreenAction::Consumed
            }
            KeyCode::Char('s') => ScreenAction::Save,
            KeyCode::Char('r') => {
                self.scanning = true;
                ScreenAction::RefreshDevices
            }
            KeyCode::Char('/') => {
                if fields.get(self.focus) == Some(&AField::Device) {
                    self.open_device_picker(true);
                }
                ScreenAction::Consumed
            }
            KeyCode::Left | KeyCode::Right => {
                if fields.get(self.focus) == Some(&AField::Buffer) {
                    let d: i8 = if key.code == KeyCode::Left { -1 } else { 1 };
                    let next = (self.staged.stream_buffer_seconds as i8 + d).clamp(1, 10);
                    self.staged.stream_buffer_seconds = next as u8;
                }
                ScreenAction::Consumed
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                let field = fields.get(self.focus).copied();
                if let Some(f) = field {
                    self.activate(f);
                }
                ScreenAction::Consumed
            }
            KeyCode::Esc => ScreenAction::Back,
            _ => ScreenAction::Consumed,
        }
    }

    /// Act on the focused field (toggle in place / open a popup). Any device
    /// re-enumeration triggered by a backend change is returned from
    /// `handle_editor_key` when the picker resolves, not here.
    fn activate(&mut self, field: AField) {
        let (_, enabled, _) = row_state(field, &self.staged);
        if !enabled && !matches!(field, AField::Backend | AField::Device) {
            return; // disabled row: inert
        }
        match field {
            AField::Backend => self.open_backend_picker(),
            AField::Device => self.open_device_picker(false),
            AField::AlsaPlugin => self.open_alsa_plugin_picker(),
            AField::Dsd => self.open_dsd_picker(),
            AField::HwVolume => self.staged.alsa_hardware_volume ^= true,
            AField::Reserve => self.staged.reserve_dac ^= true,
            AField::Exclusive => self.staged.exclusive_mode ^= true,
            AField::Passthrough => {
                self.staged.dac_passthrough ^= true;
                cascade_on_toggle(&mut self.staged, AField::Passthrough);
            }
            AField::ForceBp => self.staged.pw_force_bitperfect ^= true,
            AField::LockOutput => self.staged.skip_sink_switch ^= true,
            AField::StreamUncached => self.staged.stream_first_track ^= true,
            AField::StreamingOnly => {
                self.staged.streaming_only ^= true;
                cascade_on_toggle(&mut self.staged, AField::StreamingOnly);
            }
            AField::Buffer => {}
        }
    }

}
