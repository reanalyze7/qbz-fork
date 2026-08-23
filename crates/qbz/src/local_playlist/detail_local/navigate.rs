use slint::ComponentHandle;

use super::apply::apply;
use super::artwork_jobs::artwork_jobs;
use super::load::load;
use crate::artwork::{self, ImageCache};
use crate::local_playlist::Runtime;
use crate::{AppWindow, ContentView, NavState, PlaylistState};

/// Open a local playlist detail (the `local:` branch of
/// `navigate_playlist`). Loads + resolves off-thread, then renders through
/// the shared playlist view.
pub fn navigate(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: ImageCache,
    playlist_id: String,
) {
    handle.spawn(async move {
        let active = playlist_id.clone();
        let _ = weak.upgrade_in_event_loop(move |w| {
            crate::playlist::reset(&w);
            let state = w.global::<PlaylistState>();
            state.set_is_local(true);
            state.set_offline_only(false);
            crate::sidebar::set_active(&w, &active);
            w.global::<NavState>().set_view(ContentView::Playlist);
        });
        let Some(data) = load(&runtime, &playlist_id).await else {
            log::warn!("[qbz-slint] local playlist {playlist_id} not found");
            let _ = weak.upgrade_in_event_loop(|w| {
                w.global::<PlaylistState>().set_loading(false);
            });
            return;
        };
        let (http_jobs, local_jobs) = artwork_jobs(&data.rows);
        let _ = weak.upgrade_in_event_loop(move |w| {
            apply(&w, data);
        });
        if !http_jobs.is_empty() {
            artwork::spawn_loads(http_jobs, weak.clone(), image_cache.clone());
        }
        if !local_jobs.is_empty() {
            artwork::spawn_local_loads(local_jobs, weak.clone(), image_cache.clone());
        }
    });
}
