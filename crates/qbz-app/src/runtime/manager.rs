use std::sync::Arc;
use tokio::sync::RwLock;

use super::error::RuntimeError;
use super::requirement::CommandRequirement;
use super::{DegradedReason, RuntimeState, RuntimeStatus};

/// Runtime state manager - thread-safe, holds canonical state
pub struct RuntimeManager {
    pub(super) state: Arc<RwLock<RuntimeStatus>>,
    bootstrap_in_progress: Arc<RwLock<bool>>,
    /// Tracks which Mixtape/Collection the current queue was built from.
    /// Set by v2_enqueue_collection (replace mode), cleared by any
    /// non-Mixtape queue replacement (set_queue / clear_queue).
    /// Append-style ops (add/add_next/bulk) preserve this value.
    /// Persistence is in-memory only; session_queue_state table will
    /// add the source_collection_id column when it lands.
    queue_source_collection_id: RwLock<Option<String>>,
}

impl RuntimeManager {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(RuntimeStatus::default())),
            bootstrap_in_progress: Arc::new(RwLock::new(false)),
            queue_source_collection_id: RwLock::new(None),
        }
    }

    /// Set (or clear) which collection the current queue was built from.
    pub async fn set_queue_source_collection(&self, id: Option<String>) {
        *self.queue_source_collection_id.write().await = id;
    }

    /// Return the collection ID that seeded the current queue, if any.
    pub async fn get_queue_source_collection(&self) -> Option<String> {
        self.queue_source_collection_id.read().await.clone()
    }

    /// Get current runtime status
    pub async fn get_status(&self) -> RuntimeStatus {
        self.state.read().await.clone()
    }

    /// Check if bootstrap is in progress
    pub async fn is_bootstrap_in_progress(&self) -> bool {
        *self.bootstrap_in_progress.read().await
    }

    /// Set bootstrap in progress flag
    pub async fn set_bootstrap_in_progress(&self, in_progress: bool) {
        *self.bootstrap_in_progress.write().await = in_progress;
    }

    /// Validate command requirements against current state
    pub async fn check_requirements(&self, req: CommandRequirement) -> Result<(), RuntimeError> {
        let status = self.state.read().await;
        super::requirement::check(&status, req)
    }

    /// Check if in degraded state
    pub async fn is_degraded(&self) -> bool {
        matches!(self.state.read().await.state, RuntimeState::Degraded { .. })
    }

    /// Set degraded state
    pub async fn set_degraded(&self, reason: DegradedReason) {
        self.set_state(RuntimeState::Degraded { reason }).await;
    }
}

impl Default for RuntimeManager {
    fn default() -> Self {
        Self::new()
    }
}
