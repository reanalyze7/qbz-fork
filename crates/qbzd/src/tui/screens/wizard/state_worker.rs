// crates/qbzd/src/tui/screens/wizard/state_worker.rs — applying completed
// worker results (probes/enumeration/generation/test) onto WizardState.

use qbz_audio::{detect_distro, detect_init, detect_sandbox, AudioStackHealth, NegotiatedRate};

use crate::tui::wizard_core::{DacCandidateData, DacConfigData};

use super::state::WizardState;
use super::state_types::{Candidate, ConfigBlock};

impl WizardState {
    /// Apply a completed audio-stack probe (Check step) + detect distro/init.
    pub fn set_health(&mut self, health: AudioStackHealth) {
        self.health = Some(health);
    }

    /// Synchronously sample the cheap host descriptors when entering Check (file
    /// stats only — not the heavy shell-out probe, which runs on a worker).
    pub(super) fn sample_host(&mut self) {
        self.distro_index = detect_distro().index();
        self.init_index = detect_init().index();
        self.sandbox = detect_sandbox();
    }

    /// Apply the enumerated DAC candidates (Select-DACs step).
    pub fn set_candidates(&mut self, data: Vec<DacCandidateData>) {
        self.candidates = data.into_iter().map(Candidate::from_data).collect();
        self.detecting = false;
        self.detected = true;
        self.dac_focus = 0;
    }

    /// Apply the generated per-DAC configs (Review step).
    pub fn set_configs(&mut self, data: Vec<DacConfigData>) {
        self.configs = data
            .into_iter()
            .map(|d| ConfigBlock { data: d, flash: None })
            .collect();
        self.review_focus = 0;
        self.review_scroll = 0;
    }

    /// Apply one test read-back (Test step).
    pub fn set_test_result(
        &mut self,
        requested: Option<(u32, u32)>,
        negotiated: Option<NegotiatedRate>,
        note: Option<String>,
    ) {
        self.tested = true;
        self.test_requested = requested;
        self.test_negotiated = negotiated;
        self.test_note = note;
    }
}
