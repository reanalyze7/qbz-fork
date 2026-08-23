// crates/qbzd/src/tui/screens/audio/pickers.rs — opening the four field
// editors/popups (Backend/Device/AlsaPlugin/Dsd). `editor_input.rs` handles
// the keystrokes once one of these is open.

use qbz_audio::{AlsaPlugin, BackendManager};

use crate::tui::strings as s;
use crate::tui::widgets::SelectPopup;

use super::labels::backend_label;
use super::state::{AudioState, Editor};

impl AudioState {
    pub(super) fn open_backend_picker(&mut self) {
        let backends = BackendManager::available_backends();
        let options: Vec<String> = backends.iter().map(|b| backend_label(*b)).collect();
        let sel = backends.iter().position(|b| *b == self.staged.backend).unwrap_or(0);
        self.editor = Some(Editor::Backend(SelectPopup::new(
            s::A_BACKEND,
            options,
            sel,
            false,
        )));
    }

    pub(super) fn open_device_picker(&mut self, filter: bool) {
        let options: Vec<String> = self
            .devices
            .iter()
            .map(|d| if d.bp { format!("{} {}", d.label, s::BP_BADGE) } else { d.label.clone() })
            .collect();
        let headers: Vec<Option<String>> = self.devices.iter().map(|d| d.header.clone()).collect();
        let sel = self
            .devices
            .iter()
            .position(|d| Some(&d.id) == self.staged.output_device.as_ref() || (d.id.is_empty() && self.staged.output_device.is_none()))
            .unwrap_or(0);
        let mut popup = SelectPopup::new(s::DEVICE_PICKER_TITLE, options, sel, true).with_headers(headers);
        if filter {
            popup.filter = String::new();
        }
        self.editor = Some(Editor::Device(popup));
    }

    pub(super) fn open_alsa_plugin_picker(&mut self) {
        let opts = vec![s::ALSA_HW.to_string(), s::ALSA_PLUGHW.to_string(), s::ALSA_PCM.to_string()];
        let sel = match self.staged.alsa_plugin {
            AlsaPlugin::Hw => 0,
            AlsaPlugin::PlugHw => 1,
            AlsaPlugin::Pcm => 2,
        };
        self.editor = Some(Editor::AlsaPlugin(SelectPopup::new(s::A_ALSA_PLUGIN, opts, sel, false)));
    }

    pub(super) fn open_dsd_picker(&mut self) {
        let opts = vec![s::DSD_CONVERT.to_string(), s::DSD_DOP.to_string(), s::DSD_NATIVE.to_string()];
        let sel = match self.staged.dsd_mode.as_str() {
            "dop" => 1,
            "native" => 2,
            _ => 0,
        };
        self.editor = Some(Editor::Dsd(SelectPopup::new(s::A_DSD, opts, sel, false)));
    }
}
