//! Engine -> `OfflineState` / `SettingsState` Slint global mirroring.

use slint::ComponentHandle;

use qbz_app::offline_mode::{Connectivity, OfflineMode, OfflineStatus};
use qbz_app::user_data::UserDataPaths;

use super::engine;
use crate::{AppWindow, OfflineState, SettingsState};

/// Seed the Settings > Offline MODE toggle state from the persisted
/// engine store. Fired by the panel's `init` (`OfflineModeActions.load`),
/// so every mount of Settings > Offline re-reads it — the same lazy-load
/// hook LocalLibrarySettings uses. Best-effort: pre-session reads (no
/// store bound) keep the defaults.
pub fn seed_settings(weak: slint::Weak<AppWindow>, handle: tokio::runtime::Handle) {
    handle.spawn(async move {
        let settings = match tokio::task::spawn_blocking(|| engine().settings()).await {
            Ok(Ok(s)) => s,
            Ok(Err(e)) => {
                log::warn!("[qbz-slint] offline mode settings read failed: {e}");
                return;
            }
            Err(e) => {
                log::error!("[qbz-slint] offline mode settings seed task failed: {e}");
                return;
            }
        };
        let _ = weak.upgrade_in_event_loop(move |w| {
            w.global::<SettingsState>()
                .set_offline_mode_enabled(settings.manual_offline_mode);
        });
    });
}

/// Mirror every engine status change into the `OfflineState` Slint global
/// (login affordances + the D2 recovery banner read it). Also seeds
/// `has-previous-session` once; `enter_shell` refreshes it after a
/// successful login. Spawned once from `main` right after [`super::start`]
/// (needs the tokio runtime context and a created window).
pub fn start_ui_forwarder(weak: slint::Weak<AppWindow>) {
    let has_previous = UserDataPaths::load_last_user_id().is_some();
    let seed_weak = weak.clone();
    let _ = seed_weak.upgrade_in_event_loop(move |w| {
        w.global::<OfflineState>()
            .set_has_previous_session(has_previous);
    });

    tokio::spawn(async move {
        let mut rx = engine().subscribe();
        loop {
            let status = *rx.borrow_and_update();
            let _ = weak.upgrade_in_event_loop(move |w| apply_status(&w, status));
            if rx.changed().await.is_err() {
                break;
            }
        }
    });
}

/// Push one engine status snapshot into the Slint global (UI thread).
fn apply_status(w: &AppWindow, status: OfflineStatus) {
    let state = w.global::<OfflineState>();
    state.set_offline(status.is_offline());
    state.set_mode(match status.mode {
        OfflineMode::Online => 0,
        OfflineMode::RealOffline => 1,
        OfflineMode::InducedOffline => 2,
    });
    state.set_connectivity(match status.connectivity {
        Connectivity::Unknown => 0,
        Connectivity::Up => 1,
        Connectivity::Down => 2,
    });
    state.set_captive_portal(status.captive_portal);
    state.set_show_recovery_banner(status.show_recovery_banner());
    // The header badge's "Logged out" state needs the raw session flag —
    // show_recovery_banner() is false while connectivity is down, but the
    // badge must still read "Logged out" then.
    state.set_offline_session(status.offline_session);
    // A status broadcast resolves any in-flight Settings "Check now".
    w.global::<SettingsState>().set_offline_checking(false);
}
