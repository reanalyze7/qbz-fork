//! `navigate`: open the full-list page and load its first page.

use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use slint::{ComponentHandle, ModelRc, VecModel};

use crate::adapter::SlintAdapter;
use crate::artwork::{self, ImageCache};
use crate::{AlbumCardItem, AppWindow, ContentView, DiscoverBrowseState, NavState};

use super::fetch::{artwork_jobs, fetch_pages};
use super::filter::apply_filter;

/// Open the full-list page for `endpoint` and load its first page, then
/// fan out artwork. `genre_ids` is the shared genre-filter selection
/// (None = no filter). Mirrors `navigate_favorites` in main.rs.
pub fn navigate(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: ImageCache,
    endpoint: String,
    title: String,
    genre_ids: Option<Vec<u64>>,
) {
    let endpoint_for_fetch = endpoint.clone();
    let genre_for_fetch = genre_ids.clone();
    handle.spawn(async move {
        // Reset the page state and switch the view on the UI thread. The
        // search query is cleared on a fresh navigation; the view mode is
        // left as-is so it persists across pages.
        {
            let title = title.clone();
            let endpoint = endpoint.clone();
            let _ = weak.upgrade_in_event_loop(move |w| {
                let state = w.global::<DiscoverBrowseState>();
                state.set_title(title.into());
                state.set_endpoint(endpoint.into());
                state.set_next_offset(0);
                state.set_search_query("".into());
                state.set_albums(ModelRc::new(VecModel::from(Vec::<AlbumCardItem>::new())));
                state.set_visible(ModelRc::new(VecModel::from(Vec::<AlbumCardItem>::new())));
                state.set_loading(true);
                state.set_loading_more(false);
                state.set_has_more(true);
                w.global::<NavState>().set_view(ContentView::DiscoverBrowse);
            });
        }

        match fetch_pages(&runtime, &endpoint_for_fetch, genre_for_fetch, 0).await {
            // CardData is plain/Send — map it to the (non-Send)
            // AlbumCardItem inside the event-loop closure below. The offset
            // advances by the FETCHED count (blacklist drop is log-only).
            Ok((cards, fetched, has_more)) => {
                let jobs = artwork_jobs(&cards, 0);
                let _ = weak.upgrade_in_event_loop(move |w| {
                    let items: Vec<AlbumCardItem> =
                        cards.into_iter().map(crate::home::card_to_item).collect();
                    let state = w.global::<DiscoverBrowseState>();
                    state.set_albums(ModelRc::new(VecModel::from(items)));
                    state.set_next_offset(fetched as i32);
                    state.set_has_more(has_more);
                    state.set_loading(false);
                    apply_filter(&w);
                });
                artwork::spawn_loads(jobs, weak.clone(), image_cache.clone());
            }
            Err(e) => {
                log::error!("[qbz-slint] discover-browse load failed: {e}");
                let _ = weak.upgrade_in_event_loop(|w| {
                    let state = w.global::<DiscoverBrowseState>();
                    state.set_loading(false);
                    state.set_has_more(false);
                });
            }
        }
    });
}
