// crates/qbzd/src/tui/screens/wizard/state.rs — the WizardState struct
// definition + construction. Query/worker-result/key/draw methods live in
// sibling `state_query.rs`/`state_worker.rs`/`keys*.rs`/`draw*.rs` files as
// further `impl WizardState` blocks — fields are `pub(super)` so they stay
// reachable throughout the `wizard` module subtree without being crate-wide
// public.

use std::time::Instant;

use qbz_audio::{AudioStackHealth, NegotiatedRate, Sandbox};

use crate::tui::clipboard::ClipEnv;
use crate::tui::widgets::{SelectPopup, TextInput};

use super::state_types::{CheckField, Candidate, ConfigBlock};
use super::step::WStep;

pub struct WizardState {
    pub(super) step: WStep,
    pub(super) clip_env: ClipEnv,

    // Check step.
    pub(super) health: Option<AudioStackHealth>,
    pub(super) distro_index: usize,
    pub(super) init_index: usize,
    pub(super) sandbox: Sandbox,
    pub(super) check_focus: usize, // 0 = distro override, 1 = init override
    pub(super) check_editor: Option<(CheckField, SelectPopup)>,

    // Select-DACs step.
    pub(super) detecting: bool,
    pub(super) detected: bool,
    pub(super) candidates: Vec<Candidate>,
    pub(super) dac_focus: usize,
    pub(super) manual: Option<TextInput>,
    pub(super) manual_node: Option<String>, // last accepted, validated manual node.name
    pub(super) gate_note: Option<(String, Instant)>,

    // Review step.
    pub(super) configs: Vec<ConfigBlock>,
    pub(super) review_focus: usize,
    pub(super) review_scroll: u16,
    pub(super) status_flash: Option<(String, Instant)>,

    // Test step.
    pub(super) tested: bool,
    pub(super) test_requested: Option<(u32, u32)>, // (rate_hz, bit_depth)
    pub(super) test_negotiated: Option<NegotiatedRate>,
    pub(super) test_note: Option<String>,
}

impl WizardState {
    pub fn new() -> Self {
        WizardState {
            step: WStep::Welcome,
            clip_env: ClipEnv::from_env(),
            health: None,
            distro_index: 0,
            init_index: 0,
            sandbox: Sandbox::None,
            check_focus: 0,
            check_editor: None,
            detecting: false,
            detected: false,
            candidates: Vec::new(),
            dac_focus: 0,
            manual: None,
            manual_node: None,
            gate_note: None,
            configs: Vec::new(),
            review_focus: 0,
            review_scroll: 0,
            status_flash: None,
            tested: false,
            test_requested: None,
            test_negotiated: None,
            test_note: None,
        }
    }
}

impl Default for WizardState {
    fn default() -> Self {
        Self::new()
    }
}
