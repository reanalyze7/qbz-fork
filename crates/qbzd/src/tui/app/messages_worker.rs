// crates/qbzd/src/tui/app/messages_worker.rs — the worker-result channel's
// payload (`Msg`), the App's currently-loaded screen (`Active`), and the
// overlay/departure-target state used by the dirty-save guard.

use qbz_audio::{AudioDevice, NegotiatedRate};
use serde_json::Value;

use super::super::screens::account::AccountState;
use super::super::screens::audio::AudioState;
use super::super::screens::bundle::{BundleState, PendingImport};
use super::super::screens::network::NetworkState;
use super::super::screens::playback::PlaybackState;
use super::super::screens::scrobbler::ScrobblerState;
use super::super::screens::wizard::WizardState;
use super::super::wizard_core::{DacCandidateData, DacConfigData};
use super::messages::Screen;

pub(super) enum Msg {
    Devices(Result<Vec<AudioDevice>, String>),
    Saved { lines: Vec<String>, status: Option<Value>, reachable: bool, success: bool },
    TokenLogin(Result<(String, Option<String>), String>),
    ImportPlanned(Result<Box<PendingImport>, String>),
    ImportApplied { lines: Vec<String>, status: Option<Value>, reachable: bool },
    Exported(Result<Vec<String>, String>),
    // ---- HiFi Wizard (FB4) worker results ----
    WizardHealth(qbz_audio::AudioStackHealth),
    WizardDacs(Vec<DacCandidateData>),
    WizardConfigs(Vec<DacConfigData>),
    WizardTest {
        requested: Option<(u32, u32)>,
        negotiated: Option<NegotiatedRate>,
        note: Option<String>,
    },
}

pub(super) enum Active {
    Account(AccountState),
    Audio(AudioState),
    Playback(PlaybackState),
    Network(NetworkState),
    Bundle(BundleState),
    Wizard(WizardState),
    Scrobbler(ScrobblerState),
}

pub(super) enum Overlay {
    None,
    Help,
    Result { title: String, lines: Vec<String> },
    DirtyLeave { target: LeaveTarget },
    /// FB4: Esc mid-wizard — leaving discards the wizard's transient selections.
    ConfirmAbandon,
}

/// Where a dirty-guarded departure lands. Switching sections and quitting both
/// route through the SAME Save/Discard/Stay modal (FB3 — the modal is verbatim
/// the pre-FB3 one; only the target set changed: `Menu` became `Section`).
#[derive(Clone, Copy)]
pub(super) enum LeaveTarget {
    Section(Screen),
    Quit,
}
