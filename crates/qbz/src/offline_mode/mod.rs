//! Slint-side glue for the shared offline-MODE engine.
//!
//! Offline MODE = the app operating without Qobuz — NOT the offline CACHE
//! (downloads; that glue lives in `offline.rs` / `offline_cache.rs`). The
//! engine, connectivity actor and persisted settings are frontend-agnostic
//! (`qbz_app::offline_mode`, ADR-006); this module only owns the process
//! globals and the per-user binding, following the `tray_settings.rs`
//! template.
//!
//! It also owns the per-user `SubscriptionStateStore` binding (D4): the
//! login flows record valid/ineligible verdicts here, and the grace check
//! consults it. The purge-at-activation consumer mirrors Tauri's
//! `session_lifecycle.rs` (the Slint build never opened the store before).

mod purge;
mod subscription;
mod ui_forward;

pub use subscription::{
    init_for_user, offline_playback_allowed, subscription_mark_invalid, subscription_mark_valid,
    teardown, user_data_dir,
};
pub use ui_forward::{seed_settings, start_ui_forwarder};

use std::sync::{Arc, LazyLock, Mutex, OnceLock};

use qbz_app::offline_mode::{ConnectivityActor, OfflineModeEngine};
use qbz_app::settings::subscription::SubscriptionStateStore;

use crate::{AppWindow, SettingsState};

/// Process-global engine. Exists from first use; per-user state binds via
/// [`init_for_user`], connectivity via [`start`].
static ENGINE: LazyLock<Arc<OfflineModeEngine>> =
    LazyLock::new(|| Arc::new(OfflineModeEngine::new()));

/// The connectivity actor, spawned once per process by [`start`].
static CONNECTIVITY: OnceLock<ConnectivityActor> = OnceLock::new();

/// Per-user subscription state (D4). `None` until a session (online or
/// offline) is activated; consumers fail open in that window.
pub(super) static SUBSCRIPTION: Mutex<Option<SubscriptionStateStore>> = Mutex::new(None);

pub fn engine() -> Arc<OfflineModeEngine> {
    Arc::clone(&ENGINE)
}

/// Spawn the connectivity actor and attach it to the engine. Called once
/// from `main` after the tokio runtime is up (both spawns need the runtime
/// context); the monitoring runs for the whole app lifetime, login screen
/// included (the restore flow and the D2 recovery banner read it).
pub fn start() {
    if CONNECTIVITY.get().is_some() {
        return;
    }
    let actor = ConnectivityActor::spawn();
    engine().attach_connectivity(&actor);
    if CONNECTIVITY.set(actor).is_err() {
        log::warn!("[qbz-slint] offline mode: connectivity actor already started");
    } else {
        log::info!("[qbz-slint] offline mode: connectivity monitoring started");
    }
}

/// Force an immediate connectivity re-probe (Settings "Check now").
pub fn request_recheck() {
    if let Some(actor) = CONNECTIVITY.get() {
        actor.request_recheck();
    }
}

/// Settings > Offline "Check now": flag the in-flight state (the status
/// row's button flips to "Checking..."), then force an actor re-probe.
/// The flag clears on the next engine broadcast ([`ui_forward::apply_status`]) —
/// or after a short timeout when the verdict comes back unchanged, since the
/// actor only broadcasts state flips.
pub fn check_now(weak: slint::Weak<AppWindow>, handle: tokio::runtime::Handle) {
    use slint::ComponentHandle;
    let set_weak = weak.clone();
    let _ = set_weak.upgrade_in_event_loop(|w| {
        w.global::<SettingsState>().set_offline_checking(true);
    });
    request_recheck();
    handle.spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(4)).await;
        let _ = weak.upgrade_in_event_loop(|w| {
            w.global::<SettingsState>().set_offline_checking(false);
        });
    });
}
