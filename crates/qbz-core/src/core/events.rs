//! Event emission and raw subsystem accessors.

use std::sync::Arc;
use tokio::sync::RwLock;

use qbz_models::{CoreEvent, FrontendAdapter};
use qbz_player::QueueManager;
use qbz_qobuz::QobuzClient;

use super::QbzCore;

impl<A: FrontendAdapter + Send + Sync + 'static> QbzCore<A> {
    /// Emit an event to the frontend adapter. `pub(crate)` because every
    /// domain submodule (queue, auth, playlists, ...) needs to call this.
    pub(crate) async fn emit(&self, event: CoreEvent) {
        self.adapter.on_event(event).await;
    }

    /// Get the frontend adapter (for external event emission)
    pub fn adapter(&self) -> Arc<A> {
        Arc::clone(&self.adapter)
    }

    /// Get the Qobuz client (for advanced usage)
    pub fn client(&self) -> Arc<RwLock<Option<QobuzClient>>> {
        Arc::clone(&self.client)
    }

    /// Get the queue manager (for advanced usage)
    pub fn queue(&self) -> Arc<RwLock<QueueManager>> {
        Arc::clone(&self.queue)
    }
}
