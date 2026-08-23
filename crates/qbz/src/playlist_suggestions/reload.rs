//! Quietly reload the open Qobuz playlist detail after a mutation.

use slint::ComponentHandle;

use super::{Handle, Runtime};

/// Quietly reload the open Qobuz playlist detail after a mutation (no nav
/// record / view change — we are already on the playlist view). Refreshes the
/// track list + counts so an added suggestion shows immediately.
pub(super) fn reload_open_playlist(
    window: &crate::AppWindow,
    runtime: Runtime,
    handle: Handle,
    playlist_id: u64,
) {
    let weak = window.as_weak();
    handle.spawn(async move {
        if let Some(data) = crate::playlist::load(&runtime, playlist_id).await {
            let (http_jobs, local_jobs) = crate::playlist::artwork_jobs(&data);
            let _ = weak.upgrade_in_event_loop(move |w| {
                crate::playlist::apply(&w, data);
            });
            if let Some(cache) = crate::artwork::shared_cache() {
                if !http_jobs.is_empty() {
                    crate::artwork::spawn_loads(http_jobs, weak.clone(), cache.clone());
                }
                if !local_jobs.is_empty() {
                    crate::artwork::spawn_local_loads(local_jobs, weak.clone(), cache.clone());
                }
            }
        }
    });
}
