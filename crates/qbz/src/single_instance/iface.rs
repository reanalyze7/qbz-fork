//! D-Bus surface the primary exports for activation, independent of MPRIS
//! (which only registers after session entry — a second launch while the
//! primary sits at the login window must still raise it).

use super::present_or_defer;

pub(super) struct SingleInstanceIface;

#[zbus::interface(name = "com.blitzfc.qbz.SingleInstance")]
impl SingleInstanceIface {
    /// Raise the main window. Runs on a zbus executor thread — never touch
    /// Slint state here; `tray::present` routes through the event loop.
    fn present(&self) {
        present_or_defer();
    }

    /// A second launch carrying a Qobuz deep link forwards it here: stash
    /// the URL, present ourselves, and dispatch it through the running
    /// instance's Ctrl+L link-resolver flow (`deep_link::drain_pending`).
    /// With no shell up (sitting at the login screen, or offline) the URL
    /// stays pending for the next successful `enter_shell`.
    fn open_url(&self, url: &str) {
        crate::deep_link::stash(url.to_string());
        present_or_defer();
        crate::deep_link::drain_pending();
    }
}
