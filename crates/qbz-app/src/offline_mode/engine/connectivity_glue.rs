use std::sync::atomic::Ordering;

use super::OfflineModeEngine;
use crate::offline_mode::connectivity::{Connectivity, ConnectivityActor, ConnectivitySnapshot};
use crate::offline_mode::types::OfflineMode;
use crate::offline_mode::OfflineStatus;

impl OfflineModeEngine {
    /// Mark/unmark the session as an unauthenticated offline session
    /// ("Start offline" from the login screen). Session-scoped (D1): callers
    /// set it on `enter_shell_offline` and clear it after a successful login.
    pub fn set_offline_session(&self, active: bool) {
        self.offline_session.store(active, Ordering::Relaxed);
        self.recompute();
    }

    /// Feed a fresh connectivity snapshot (the engine's listener task calls
    /// this on every actor broadcast).
    pub fn on_connectivity(&self, snapshot: ConnectivitySnapshot) {
        if let Ok(mut guard) = self.connectivity.lock() {
            *guard = snapshot;
        }
        self.recompute();
    }

    /// Spawn the listener wiring an actor subscription into the engine.
    /// Returns immediately; the task lives for the process lifetime.
    pub fn attach_connectivity(self: &std::sync::Arc<Self>, actor: &ConnectivityActor) {
        let mut rx = actor.subscribe();
        let engine = std::sync::Arc::clone(self);
        tokio::spawn(async move {
            loop {
                if rx.changed().await.is_err() {
                    return;
                }
                let snapshot = *rx.borrow();
                engine.on_connectivity(snapshot);
            }
        });
    }

    /// Derive the mode, flip the Qobuz gate, broadcast on change.
    pub(super) fn recompute(&self) {
        let induced = self.induced.load(Ordering::Relaxed);
        let offline_session = self.offline_session.load(Ordering::Relaxed);
        let connectivity = self
            .connectivity
            .lock()
            .map(|guard| *guard)
            .unwrap_or_default();

        let mode = if induced {
            OfflineMode::InducedOffline
        } else if offline_session || connectivity.state == Connectivity::Down {
            OfflineMode::RealOffline
        } else {
            OfflineMode::Online
        };

        let status = OfflineStatus {
            mode,
            connectivity: connectivity.state,
            captive_portal: connectivity.captive_portal,
            induced,
            offline_session,
        };

        // D3: the single Qobuz choke point follows the mode.
        qbz_qobuz::offline_gate::set_offline(mode != OfflineMode::Online);

        let _ = self.status_tx.send_if_modified(|current| {
            if *current != status {
                log::info!(
                    "[OfflineMode] {:?} -> {:?} (connectivity {:?}, induced {}, offline_session {})",
                    current.mode,
                    status.mode,
                    status.connectivity,
                    status.induced,
                    status.offline_session
                );
                *current = status;
                true
            } else {
                false
            }
        });
    }
}
