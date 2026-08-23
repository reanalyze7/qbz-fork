use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use qbz_app::playback_driver;
use qbz_models::CoreEvent;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use crate::adapter::DaemonAdapter;

/// T10 (§7.5): the queue-persistence subscriber. Debounces `CoreEvent::QueueUpdated`
/// bursts by 2 s, then flushes the live queue + position to the session store via
/// `save_session_now`. QConnect-driven mutations (`materialize_remote_queue` ->
/// `set_queue`) also emit `QueueUpdated`, so a remote-set queue survives a restart
/// (boot restores it PAUSED). Non-queue events (e.g. position ticks) are drained
/// WITHOUT extending the debounce window, so they can never starve the flush.
pub(super) fn spawn_queue_persist(
    runtime: Arc<AppRuntime<DaemonAdapter>>,
    mut rx: broadcast::Receiver<CoreEvent>,
) -> JoinHandle<()> {
    use tokio::sync::broadcast::error::RecvError;
    const DEBOUNCE: std::time::Duration = std::time::Duration::from_secs(2);
    tokio::spawn(async move {
        loop {
            // Block until the FIRST queue mutation of a burst.
            match rx.recv().await {
                Ok(CoreEvent::QueueUpdated { .. }) => {}
                Ok(_) => continue,
                Err(RecvError::Lagged(_)) => continue,
                Err(RecvError::Closed) => return,
            }
            // Debounce: a fixed deadline that only a further QueueUpdated extends.
            // Other events are consumed but never push the deadline out.
            let mut deadline = tokio::time::Instant::now() + DEBOUNCE;
            loop {
                tokio::select! {
                    _ = tokio::time::sleep_until(deadline) => break,
                    r = rx.recv() => match r {
                        Ok(CoreEvent::QueueUpdated { .. }) => {
                            deadline = tokio::time::Instant::now() + DEBOUNCE;
                        }
                        Ok(_) => {}
                        Err(RecvError::Lagged(_)) => {}
                        Err(RecvError::Closed) => return,
                    }
                }
            }
            playback_driver::save_session_now(runtime.as_ref()).await;
            log::debug!("[qbzd] queue-persist: session flushed after QueueUpdated burst");
        }
    })
}
