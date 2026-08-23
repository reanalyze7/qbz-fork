use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Confirmation burst delays after a failed probe while `Up` (verify before
/// flipping down — replaces Tauri's "2 polls ~60 s" with ~10 s of focused
/// re-checking that can't be gamed by unspaced extra polls).
pub(super) const CONFIRM_DELAYS: [Duration; 2] = [Duration::from_secs(3), Duration::from_secs(7)];

/// Raw connectivity verdict, independent of the app's offline MODE.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Connectivity {
    Up,
    Down,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConnectivitySnapshot {
    pub state: Connectivity,
    /// A probe was answered by a redirect — typical captive portal. Surfaced
    /// as a hint; the state is still `Down` (D3: a portal cannot reach Qobuz).
    pub captive_portal: bool,
}

impl Default for ConnectivitySnapshot {
    fn default() -> Self {
        Self {
            state: Connectivity::Unknown,
            captive_portal: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeOutcome {
    Success,
    /// All endpoints failed (timeout / error / wrong payload).
    Failure,
    /// At least one endpoint answered with a redirect and none succeeded.
    CaptivePortal,
}

/// What the actor should do after feeding the judge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JudgeAction {
    /// Nothing pending — sleep until the next regular tick.
    Idle,
    /// Re-probe after the given delay (confirmation burst step).
    ConfirmAfter(Duration),
}
