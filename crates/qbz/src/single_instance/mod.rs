//! Single-instance guard (issues #544/#559 — Tauri parity: the old app
//! shipped tauri-plugin-single-instance; the Slint rebuild lost it, so
//! every click on a pinned taskbar shortcut / launcher entry spawned
//! another full player — reported on both Hyprland and KDE).
//!
//! The first instance takes ownership of the well-known session-bus name
//! `io.github.reanalyze7.qoqobuz` (Flatpak auto-grants owning the app-id name — no
//! finish-args change needed) and exports a `io.github.reanalyze7.qoqobuz.SingleInstance`
//! interface with `Present()` and `OpenUrl(url)` methods. A second launch
//! sees the name taken and calls `OpenUrl(url)` when its own argv carried a
//! Qobuz deep link (the primary presents itself AND navigates — Tauri
//! parity, the piece #618 didn't port) or `Present()` otherwise — which
//! raises the main window — and works from process start, login screen
//! included — and exits. If the primary predates the interface (≤2.0.x) the
//! call errors and the second launch falls back to the MPRIS `Raise`
//! method, which only exists after session entry. `OpenUrl` failing on an
//! older primary (interface present, method missing) falls back to bare
//! `Present()`. Any D-Bus problem — no session bus, weird sandbox — falls
//! through as "we are primary": the guard must never block startup.
//!
//! Blocking zbus API on purpose: this runs once on the main thread before
//! the UI exists, and the async-io executor self-drives the connection
//! from any context (the zbus 5 "tokio" feature is FORBIDDEN graph-wide —
//! see the rfd/ksni comments in Cargo.toml).
#![cfg(target_os = "linux")]

mod iface;
mod probe;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

use zbus::blocking::Connection;

use crate::AppWindow;

const BUS_NAME: &str = "io.github.reanalyze7.qoqobuz";
const OBJECT_PATH: &str = "/io/github/reanalyze7/qoqobuz";
const IFACE_NAME: &str = "io.github.reanalyze7.qoqobuz.SingleInstance";

/// Keeps the acquired name owned for the process lifetime (releasing it
/// would let a second launch believe it is primary).
static CONN: std::sync::OnceLock<Connection> = std::sync::OnceLock::new();

/// The main window, published by `bind_window` right after `AppWindow::new()`
/// so `Present()` can raise it. `slint::Weak` is Send+Sync; upgrades happen
/// on the event loop (`tray::present` hops there itself).
static MAIN_WEAK: OnceLock<slint::Weak<AppWindow>> = OnceLock::new();

/// A `Present()` arrived before the window existed (simultaneous cold starts:
/// the DoNotQueue loser can call in while the winner is still initializing).
/// Drained once by `bind_window`.
static PENDING_PRESENT: AtomicBool = AtomicBool::new(false);

/// Shared Present path for the iface methods: raise whichever window is
/// current, or remember the request until `bind_window` (simultaneous cold
/// starts: the DoNotQueue loser can call in while the winner is still
/// initializing).
pub(super) fn present_or_defer() {
    match MAIN_WEAK.get() {
        Some(weak) => crate::tray::present(weak),
        None => PENDING_PRESENT.store(true, Ordering::SeqCst),
    }
}

/// Publish the main window to the `Present()` handler. Call right after
/// `AppWindow::new()`; drains a Present that landed before the window existed.
pub fn bind_window(weak: slint::Weak<AppWindow>) {
    let _ = MAIN_WEAK.set(weak);
    if PENDING_PRESENT.swap(false, Ordering::SeqCst) {
        if let Some(weak) = MAIN_WEAK.get() {
            crate::tray::present(weak);
        }
    }
}

/// True = we are the primary instance (name acquired, or D-Bus unusable).
/// False = another instance owns the name; it has been asked to raise its
/// window and the caller should exit.
pub fn acquire_or_raise() -> bool {
    match probe::probe() {
        Ok(primary) => primary,
        Err(e) => {
            log::warn!(
                "[qbz-slint] single-instance: D-Bus probe failed ({e}); continuing as primary"
            );
            true
        }
    }
}
