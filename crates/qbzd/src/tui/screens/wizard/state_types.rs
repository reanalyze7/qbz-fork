// crates/qbzd/src/tui/screens/wizard/state_types.rs — the small value types
// backing WizardState's per-step sub-state.

use std::time::Instant;

use crate::tui::clipboard::Tier;
use crate::tui::wizard_core::{DacCandidateData, DacConfigData};

/// Which Check-step override the select popup is editing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CheckField {
    Distro,
    Init,
}

/// A DAC row on the Select-DACs step (checkbox + probed rates).
#[derive(Debug, Clone)]
pub(super) struct Candidate {
    pub(super) id: String,
    pub(super) description: String,
    pub(super) bus: String,
    pub(super) is_default: bool,
    pub(super) looks_like_dac: bool,
    pub(super) rates_label: String,
    pub(super) checked: bool,
}

impl Candidate {
    pub(super) fn from_data(d: DacCandidateData) -> Self {
        Candidate {
            checked: d.looks_like_dac, // pre-select the likely DACs
            id: d.id,
            description: d.description,
            bus: d.bus,
            is_default: d.is_default,
            looks_like_dac: d.looks_like_dac,
            rates_label: d.rates_label,
        }
    }
}

/// A generated config + its per-block copy flash. The flash stores which
/// `Tier` won so the render can pick the right wording (OSC 52's is
/// deliberately not "copied ✓" — see `clipboard::Tier::short_label`).
pub(super) struct ConfigBlock {
    pub(super) data: DacConfigData,
    pub(super) flash: Option<(Tier, Instant)>,
}
