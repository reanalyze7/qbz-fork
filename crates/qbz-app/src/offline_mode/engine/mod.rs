mod connectivity_glue;
mod lifecycle;
mod settings_ops;

use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use tokio::sync::watch;

use super::connectivity::ConnectivitySnapshot;
use super::store::OfflineModeStore;
use super::types::{default_status, OfflineStatus};

/// The engine. One per process; frontends hold it in an `Arc`.
pub struct OfflineModeEngine {
    pub(super) store: Mutex<Option<OfflineModeStore>>,
    pub(super) induced: AtomicBool,
    pub(super) offline_session: AtomicBool,
    pub(super) status_tx: watch::Sender<OfflineStatus>,
    pub(super) connectivity: Mutex<ConnectivitySnapshot>,
}

impl OfflineModeEngine {
    pub fn new() -> Self {
        Self {
            store: Mutex::new(None),
            induced: AtomicBool::new(false),
            offline_session: AtomicBool::new(false),
            status_tx: watch::channel(default_status()).0,
            connectivity: Mutex::new(ConnectivitySnapshot::default()),
        }
    }

    /// Subscribe to status changes (UI listeners, QConnect suppressor, ...).
    pub fn subscribe(&self) -> watch::Receiver<OfflineStatus> {
        self.status_tx.subscribe()
    }

    /// Current status snapshot.
    pub fn status(&self) -> OfflineStatus {
        *self.status_tx.borrow()
    }

    /// Convenience: is ANY offline flavor active?
    pub fn is_offline(&self) -> bool {
        self.status().is_offline()
    }
}

impl Default for OfflineModeEngine {
    fn default() -> Self {
        Self::new()
    }
}
