//! Per-shell context binding: the gate that lets `drain_pending` dispatch
//! (session active, `AppWindow` alive).

use std::sync::{Arc, Mutex};

use qbz_app::shell::AppRuntime;

use crate::adapter::SlintAdapter;
use crate::artwork;
use crate::AppWindow;

use super::dispatch::dispatch;
use super::take_pending;

/// Everything `dispatch` needs, bound once per shell entry (`enter_shell`)
/// and cleared on logout so "context set" means "a session is active and
/// navigation can succeed". The warm D-Bus path has no other way to reach
/// these — they only exist as locals in `main()`.
#[derive(Clone)]
pub(super) struct ShellCtx {
    pub(super) runtime: Arc<AppRuntime<SlintAdapter>>,
    pub(super) weak: slint::Weak<AppWindow>,
    pub(super) handle: tokio::runtime::Handle,
    pub(super) image_cache: artwork::ImageCache,
}

static SHELL_CTX: Mutex<Option<ShellCtx>> = Mutex::new(None);

/// Bind the shell context at `enter_shell` — the gate that lets
/// `drain_pending` dispatch (session active, AppWindow alive).
pub fn bind_shell_ctx(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    image_cache: artwork::ImageCache,
) {
    if let Ok(mut guard) = SHELL_CTX.lock() {
        *guard = Some(ShellCtx {
            runtime,
            weak,
            handle,
            image_cache,
        });
    }
}

/// Clear the shell context on logout: back at the login screen a pending
/// URL must wait for the next `enter_shell`, not fire into a dead session.
pub fn clear_shell_ctx() {
    if let Ok(mut guard) = SHELL_CTX.lock() {
        *guard = None;
    }
}

/// Dispatch the pending URL through the existing Ctrl+L resolve flow, but
/// only when a shell is up. No-op otherwise — the URL stays pending for the
/// next successful `enter_shell`. Safe to call from any thread (the zbus
/// executor included): the resolve spawns on the stored tokio handle and
/// the navigation hops to the Slint event loop.
pub fn drain_pending() {
    let ctx = SHELL_CTX.lock().ok().and_then(|guard| guard.clone());
    let Some(ctx) = ctx else { return };
    let Some(url) = take_pending() else { return };
    dispatch(url, ctx);
}
