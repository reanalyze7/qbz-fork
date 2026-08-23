//! Open the Playlist Manager and load its data. Mirrors `navigate_favorites`.

use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use slint::ComponentHandle;

use crate::adapter::SlintAdapter;
use crate::artwork::{self, ImageCache};
use crate::{AppWindow, ContentView, NavState};

use super::artwork::{artwork_jobs, load_folder_custom_images};
use super::load::load;
use super::render::{apply, reset_session, set_loading};

pub fn navigate(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: ImageCache,
) {
    handle.spawn(async move {
        let _ = weak.upgrade_in_event_loop(|w| {
            reset_session(&w);
            set_loading(&w, true);
            w.global::<NavState>().set_view(ContentView::PlaylistManager);
        });
        let data = load(&runtime).await;
        let handle2 = tokio::runtime::Handle::current();
        let weak2 = weak.clone();
        let _ = weak.upgrade_in_event_loop(move |w| {
            apply(&w, data);
            let jobs = artwork_jobs(&w);
            artwork::spawn_loads(jobs, weak2.clone(), image_cache.clone());
            load_folder_custom_images(weak2.clone(), &handle2);
        });
    });
}
