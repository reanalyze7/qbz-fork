//! `load_more`: fetch the next page and append it to the grid.

use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use slint::{ComponentHandle, Model, ModelRc, VecModel};

use crate::adapter::SlintAdapter;
use crate::artwork::{self, ImageCache};
use crate::home::CardData;
use crate::{AlbumCardItem, AppWindow, DiscoverBrowseState};

use super::fetch::{artwork_jobs, fetch_pages};
use super::filter::apply_filter;

/// Fetch the next page (offset = DiscoverBrowseState.next-offset) and
/// append it to the grid. Wired to DiscoverBrowseActions::load-more.
/// `genre_ids` is the shared genre-filter selection (None = no filter).
pub fn load_more(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: ImageCache,
    genre_ids: Option<Vec<u64>>,
) {
    let Some(w) = weak.upgrade() else {
        return;
    };
    let state = w.global::<DiscoverBrowseState>();
    if !state.get_has_more() || state.get_loading_more() {
        return;
    }
    // A non-empty search filters the loaded set client-side; pulling more
    // pages while filtering matches no UX (Tauri disables load-more too).
    if !state.get_search_query().is_empty() {
        return;
    }
    let endpoint = state.get_endpoint().to_string();
    let offset = state.get_next_offset().max(0) as u32;
    // New cards land after the currently-loaded albums. With narrowing
    // active the server offset outruns the model length, so the artwork
    // base index must come from the model, not the offset.
    let base_index = state.get_albums().row_count();
    state.set_loading_more(true);

    handle.spawn(async move {
        match fetch_pages(&runtime, &endpoint, genre_ids, offset).await {
            Ok((cards, fetched, has_more)) => {
                let jobs = artwork_jobs(&cards, base_index);
                let _ = weak.upgrade_in_event_loop(move |w| {
                    append_albums(&w, cards, fetched, has_more);
                });
                artwork::spawn_loads(jobs, weak.clone(), image_cache.clone());
            }
            Err(e) => {
                log::error!("[qbz-slint] discover-browse load-more failed: {e}");
                let _ = weak.upgrade_in_event_loop(|w| {
                    w.global::<DiscoverBrowseState>().set_loading_more(false);
                });
            }
        }
    });
}

/// Append a freshly-fetched page onto the existing grid, advancing the
/// offset by the FETCHED item count (`cards` may be a narrowed subset)
/// and updating has-more. UI thread only.
fn append_albums(window: &AppWindow, cards: Vec<CardData>, fetched: u32, has_more: bool) {
    let state = window.global::<DiscoverBrowseState>();
    let model = state.get_albums();
    let mut combined: Vec<AlbumCardItem> = (0..model.row_count())
        .filter_map(|i| model.row_data(i))
        .collect();
    combined.extend(cards.into_iter().map(crate::home::card_to_item));
    state.set_albums(ModelRc::new(VecModel::from(combined)));
    state.set_next_offset(state.get_next_offset() + fetched as i32);
    state.set_has_more(has_more);
    state.set_loading_more(false);
    apply_filter(window);
}
