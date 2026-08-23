use qbz_audio::settings::AudioSettings;
use qbz_audio::{AudioBackendType, AudioDevice};

use crate::tui::strings as s;
use crate::tui::widgets::SelectPopup;

use super::device_grouping::{group_devices, DeviceEntry};
use super::fields::{visible_fields, AField};
use super::model::StagedAudio;

// ============================ screen state ============================

pub(super) enum Editor {
    Backend(SelectPopup),
    Device(SelectPopup),
    AlsaPlugin(SelectPopup),
    Dsd(SelectPopup),
    /// The §3.2.4 DSD guard: `prev` is restored on Esc.
    DsdConfirm { new: String, prev: String },
}

pub struct AudioState {
    pub(super) baseline: StagedAudio,
    pub(super) staged: StagedAudio,
    pub(super) focus: usize,
    pub(super) devices: Vec<DeviceEntry>,
    pub(super) scanning: bool,
    pub(super) editor: Option<Editor>,
}

impl AudioState {
    pub fn new(settings: &AudioSettings) -> Self {
        let staged = StagedAudio::from_settings(settings);
        Self {
            baseline: staged.clone(),
            staged,
            focus: 0,
            devices: Vec::new(),
            scanning: true,
            editor: None,
        }
    }

    /// The backend the App should (re-)enumerate devices for.
    pub fn backend(&self) -> AudioBackendType {
        self.staged.backend
    }

    /// Receive a device-enumeration result from the worker (§5.5).
    pub fn set_devices(&mut self, result: Result<Vec<AudioDevice>, String>) {
        self.scanning = false;
        self.devices = match result {
            Ok(list) => group_devices(self.staged.backend, list),
            Err(_) => group_devices(self.staged.backend, Vec::new()),
        };
    }

    pub fn start_scan(&mut self) {
        self.scanning = true;
    }

    pub fn is_dirty(&self) -> bool {
        self.staged != self.baseline
    }

    pub fn is_editing(&self) -> bool {
        self.editor.is_some()
    }

    /// The breadcrumb's level-2 node when a field editor/picker is active (the
    /// DSD guard counts — it is still editing the DSD field).
    pub fn editing_label(&self) -> Option<&'static str> {
        match &self.editor {
            Some(Editor::Backend(_)) => Some(s::A_BACKEND),
            Some(Editor::Device(_)) => Some(s::A_DEVICE),
            Some(Editor::AlsaPlugin(_)) => Some(s::A_ALSA_PLUGIN),
            Some(Editor::Dsd(_)) | Some(Editor::DsdConfirm { .. }) => Some(s::A_DSD),
            None => None,
        }
    }

    /// True when the focused (non-editing) field consumes ←/→ (the Buffer
    /// slider). The shell asks this before letting ← drop focus back to the nav.
    pub fn focused_is_buffer(&self) -> bool {
        if self.editor.is_some() {
            return false;
        }
        let fields = visible_fields(&self.staged);
        fields.get(self.focus).copied() == Some(AField::Buffer)
    }

    pub fn mark_saved(&mut self) {
        self.baseline = self.staged.clone();
    }
}
