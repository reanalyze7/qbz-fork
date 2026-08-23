//! XDG / launcher deep links — `xdg-open https://open.qobuz.com/album/<id>`
//! with the desktop files' `Exec=qbz %u` (Tauri parity: the old app scanned
//! argv in the single-instance plugin callback and on cold start; the Slint
//! rebuild kept the raise-half but dropped the URL-half, so a deep link
//! raised/focused the window without ever navigating).
//!
//! Two entry paths, one pending slot:
//!
//! - **Cold start:** `capture_argv()` at the top of `main()` stashes the
//!   first Qobuz-link argv entry BEFORE the single-instance guard runs (a
//!   second launch must read its own argv before deciding to raise+exit).
//!   Drained at the END of `enter_shell` — after the startup-page/view
//!   restore, so the restore can't re-root over the deep link. At that point
//!   the session is active and the AppWindow exists, so NO sleep hack (the
//!   Tauri 1500 ms delay is not ported). Sitting at the login screen (no
//!   session) the URL simply stays pending until the next successful
//!   `enter_shell`. Offline entry (`enter_shell_offline`) never binds the
//!   shell context, so the URL rides until an online shell — navigation
//!   needs the API (same limitation as the Tauri era).
//! - **Warm start:** the second launch forwards the URL over the
//!   single-instance D-Bus interface (`OpenUrl`, see `single_instance.rs`),
//!   which presents the running instance and drains through the same path.
//!
//! The dispatch itself is the EXISTING Ctrl+L machinery:
//! `link_resolver::resolve` → `apply_resolved_link` → `navigate_*`.

use std::sync::Mutex;

mod dispatch;
mod shell_ctx;

#[cfg(test)]
mod tests;

pub use shell_ctx::{bind_shell_ctx, clear_shell_ctx, drain_pending};

/// The first Qobuz link seen, waiting for a shell to navigate it. A warm
/// `OpenUrl` overwrites: the newest user intent wins.
static PENDING: Mutex<Option<String>> = Mutex::new(None);

/// Whether a string looks like a Qobuz link (custom scheme or web URL) —
/// 1:1 the legacy Tauri `is_qobuz_link` prefixes
/// (`qbz-worktrees/legacy-tauri` `src-tauri/src/lib.rs:594`).
pub fn is_qobuz_link(arg: &str) -> bool {
    arg.starts_with("qobuzapp://")
        || arg.starts_with("https://play.qobuz.com/")
        || arg.starts_with("http://play.qobuz.com/")
        || arg.starts_with("https://open.qobuz.com/")
        || arg.starts_with("http://open.qobuz.com/")
}

/// The first Qobuz link in an argument list, if any (pure — unit-tested).
fn select_link(args: &[String]) -> Option<String> {
    args.iter().find(|a| is_qobuz_link(a)).cloned()
}

/// Scan the process argv for a Qobuz link and stash it pending. Call at the
/// top of `main()`, BEFORE `single_instance::acquire_or_raise`: when another
/// instance owns the bus name the guard forwards the stashed URL to it, and
/// when we are the primary it rides until the `enter_shell` drain.
pub fn capture_argv() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(url) = select_link(&args) {
        log::info!(
            "[qbz-slint] deep link: captured from argv: {}",
            url.split('?').next().unwrap_or(&url)
        );
        stash(url);
    }
}

/// Stash a URL pending (cold argv capture and the warm D-Bus `OpenUrl`).
pub fn stash(url: String) {
    if let Ok(mut guard) = PENDING.lock() {
        *guard = Some(url);
    }
}

/// Take the pending URL, leaving the slot empty.
pub fn take_pending() -> Option<String> {
    PENDING.lock().ok().and_then(|mut guard| guard.take())
}
