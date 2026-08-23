// crates/qbzd/src/tui/app/messages.rs — the shared vocabulary the App state
// machine and the screens communicate through: `Screen`, the intent a
// screen's key handler returns (`ScreenAction`), what the event loop must do
// post-key (`LoopCmd`), and the read-only render context (`DrawCtx`). The
// worker-result channel's payload (`Msg`) and the App's internal
// `Active`/`Overlay`/`LeaveTarget` state live in `messages_worker.rs`.

use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Account,
    Audio,
    Playback,
    Network,
    Bundle,
    Wizard,
    Scrobbler,
}

/// Seven sections. The original D7 six-screen cap was broken deliberately for
/// FB4's HiFi Wizard, and again for the CONSOLE ext's Scrobbler (Last.fm /
/// ListenBrainz auth) — both owner-sanctioned. Scrobbler is appended LAST so no
/// existing section index (or number-key jump) shifts.
pub const SCREENS: [Screen; 7] = [
    Screen::Account,
    Screen::Audio,
    Screen::Playback,
    Screen::Network,
    Screen::Bundle,
    Screen::Wizard,
    Screen::Scrobbler,
];

/// The intent a screen's key handler returns to the App.
pub enum ScreenAction {
    Consumed,
    Save,
    Back,
    RefreshDevices,
    LoginBrowser,
    LoginToken(String),
    Logout,
    ImportPlan(String),
    ImportApply,
    Export { dest: String, include_auth: bool },
    // ---- HiFi Wizard (FB4) worker requests ----
    /// Run the heavy audio-stack health probe (Check step).
    WizardProbeHealth,
    /// Enumerate DAC candidates via pw-dump (Select-DACs step).
    WizardDetect,
    /// Generate the per-DAC config blocks (Review step).
    WizardGenConfigs(Vec<(String, String)>),
    /// Start the daemon's own playback of the current queue, then read back.
    WizardTestStart,
    /// Re-read the requested-vs-negotiated rate without (re)starting playback.
    WizardTestPoll,
    /// Esc mid-wizard — open the confirm-abandon modal.
    WizardAbandon,
    // ---- Scrobbler (CONSOLE ext) ----
    /// Suspend the alt-screen and run the Last.fm connect flow on the plain
    /// terminal (same methodology as the browser login).
    ScrobbleConnectLastfm,
    /// Suspend the alt-screen and run the ListenBrainz token connect flow.
    ScrobbleConnectListenbrainz,
}

/// Read-only context passed to every screen's `draw` (the live status body for
/// the screens that render a daemon-state line).
pub struct DrawCtx<'a> {
    pub status: Option<&'a Value>,
}

/// What the event loop must do after handling a key (terminal-control cases).
pub enum LoopCmd {
    None,
    /// Suspend the alt-screen and run the T5 browser-login engine on the plain
    /// terminal, then resume (see the task report for this deliberate divergence).
    BrowserLogin,
    /// Suspend the alt-screen and run the Last.fm scrobbler connect flow.
    ScrobbleLastfm,
    /// Suspend the alt-screen and run the ListenBrainz scrobbler connect flow.
    ScrobbleListenbrainz,
}
