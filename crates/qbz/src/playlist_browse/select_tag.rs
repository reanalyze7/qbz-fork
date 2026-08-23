//! Tag switching (radio-flag update + re-fetch page 0).

use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use slint::{ComponentHandle, Model, ModelRc, VecModel};

use super::artwork::artwork_jobs;
use super::fetch::fetch_page;
use super::filter::apply_filter;
use super::model::to_item;
use super::SELECTED_TAG;
use crate::adapter::SlintAdapter;
use crate::artwork::{self, ImageCache};
use crate::{AppWindow, PlaylistBrowseState, SearchPlaylistItem};

/// Select a category tag (slug; "" = All): update the radio flags, then
/// re-fetch page 0 server-side with the tag + the shared genre selection
/// (same as `navigate` minus the view switch and the tag re-fetch). The
/// search query is kept — it is a client-side filter over whatever set is
/// loaded (Tauri keeps it across tag switches too). UI thread entry.
pub fn select_tag(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: ImageCache,
    slug: String,
    genre_ids: Option<Vec<u64>>,
) {
    {
        let Ok(mut s) = SELECTED_TAG.lock() else {
            return;
        };
        if *s == slug {
            // Re-clicking the active tag (or All) is a no-op.
            return;
        }
        *s = slug.clone();
    }
    let Some(w) = weak.upgrade() else {
        return;
    };
    let state = w.global::<PlaylistBrowseState>();
    state.set_selected_tag(slug.clone().into());
    let tags = state.get_tags();
    for i in 0..tags.row_count() {
        if let Some(mut t) = tags.row_data(i) {
            let sel = t.slug.as_str() == slug;
            if t.selected != sel {
                t.selected = sel;
                tags.set_row_data(i, t);
            }
        }
    }
    // Reset the pagination and reload page 0.
    state.set_next_offset(0);
    state.set_playlists(ModelRc::new(VecModel::from(
        Vec::<SearchPlaylistItem>::new(),
    )));
    state.set_visible(ModelRc::new(VecModel::from(
        Vec::<SearchPlaylistItem>::new(),
    )));
    state.set_loading(true);
    state.set_loading_more(false);
    state.set_has_more(true);

    handle.spawn(async move {
        match fetch_page(&runtime, &slug, genre_ids, 0).await {
            Ok((cards, has_more)) => {
                let jobs = artwork_jobs(&cards, 0);
                let _ = weak.upgrade_in_event_loop(move |w| {
                    let items: Vec<SearchPlaylistItem> = cards.iter().map(to_item).collect();
                    let fetched = items.len() as i32;
                    let state = w.global::<PlaylistBrowseState>();
                    state.set_playlists(ModelRc::new(VecModel::from(items)));
                    state.set_next_offset(fetched);
                    state.set_has_more(has_more);
                    state.set_loading(false);
                    apply_filter(&w);
                });
                artwork::spawn_loads(jobs, weak.clone(), image_cache.clone());
            }
            Err(e) => {
                log::error!("[qbz-slint] playlist-browse tag load failed: {e}");
                let _ = weak.upgrade_in_event_loop(|w| {
                    let state = w.global::<PlaylistBrowseState>();
                    state.set_loading(false);
                    state.set_has_more(false);
                });
            }
        }
    });
}
