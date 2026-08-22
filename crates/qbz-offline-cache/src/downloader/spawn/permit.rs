//! Acquire the download-concurrency semaphore permit, reporting failure
//! through the DB status + `CacheEventSink` the same way any other
//! early-abort path does.

use crate::event::{CacheEvent, CacheEventSink};

pub(super) async fn acquire_or_report(
    track_id: u64,
    db: &std::sync::Arc<tokio::sync::Mutex<Option<crate::OfflineCacheDb>>>,
    sink: &CacheEventSink,
    semaphore: &std::sync::Arc<tokio::sync::Semaphore>,
) -> Option<tokio::sync::OwnedSemaphorePermit> {
    match semaphore.clone().acquire_owned().await {
        Ok(permit) => Some(permit),
        Err(err) => {
            log::error!(
                "Failed to acquire cache slot for track {}: {}",
                track_id,
                err
            );
            if let Some(db_guard) = db.lock().await.as_ref() {
                let _ = db_guard.update_status(
                    track_id,
                    crate::OfflineCacheStatus::Failed,
                    Some("Failed to start caching"),
                );
            }
            sink(CacheEvent::Failed {
                track_id,
                error: "Failed to acquire cache slot".to_string(),
            });
            None
        }
    }
}
