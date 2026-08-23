// crates/qbzd/src/mpris/ — MPRIS system media controls (CONSOLE ext).
//
// Publishes the daemon's playback over the standard org.mpris.MediaPlayer2
// D-Bus interface via qbz-media-controls (mpris-server on Linux, INCLUDING the
// DesktopEntry that lets GNOME/KDE resolve the app icon). This is what makes a
// KDE Plasma media widget — or a plasmoid — control the daemon with NO custom
// client code, and makes hardware media keys work.
//
// Two halves:
//   * OUTBOUND — a CoreEvent-bus subscriber pushes now-playing metadata plus
//     play/pause/position/volume into the OS controls.
//   * INBOUND — the qbz-media-controls callback maps MediaEvent (media keys,
//     the desktop widget) back onto core transport commands.
//
// The inbound callback holds only a Weak<AppRuntime> (upgraded per event), and
// the updater task upgrades a Weak once to seed then drops it — so the
// integration NEVER pins the runtime in steady state and shutdown ordering
// (#521: the audio device must release before drop(booted)) is unaffected.
//
// Enablement: on by default where a session bus exists; `QBZD_MPRIS` in
// {0,false,off,no} disables it. On a headless server with no D-Bus, `spawn`
// returns None gracefully even when enabled — the daemon runs fine without it.
mod inbound;
mod mapping;
mod spawn;
#[cfg(test)]
mod tests;

use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use qbz_media_controls::MediaIntegration;
use tokio::task::JoinHandle;

use crate::adapter::DaemonAdapter;
use crate::paths::ProfileRoots;

pub use spawn::spawn;

type Runtime = Arc<AppRuntime<DaemonAdapter>>;

/// A running MPRIS integration: the OS-controls handle (kept alive so the D-Bus
/// service stays published) plus the bus→controls updater task.
pub struct MprisHandle {
    integration: Arc<dyn MediaIntegration>,
    updater: JoinHandle<()>,
}

impl MprisHandle {
    /// Abort the updater and drop the OS-controls handle (tears down the D-Bus
    /// service). The inbound callback held only a Weak<AppRuntime>, so this does
    /// not participate in the #521 audio-release ordering.
    pub async fn shutdown(self) {
        self.updater.abort();
        let _ = self.updater.await;
        drop(self.integration);
    }
}

/// Whether MPRIS should be published. The `QBZD_MPRIS` env var wins when set
/// (deploy/override knob); otherwise the persisted `daemon_prefs.mpris_enabled`
/// toggle decides (default ON), which is what the setup-TUI Playback screen and
/// `qbzd settings set playback.mpris` write.
fn enabled(roots: &ProfileRoots) -> bool {
    if let Ok(v) = std::env::var("QBZD_MPRIS") {
        return !matches!(v.trim().to_ascii_lowercase().as_str(), "0" | "false" | "off" | "no");
    }
    qbz_app::settings::daemon_prefs::load_at(&roots.data).mpris_enabled
}
