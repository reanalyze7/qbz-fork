//! Pagination continuation.

use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use slint::{ComponentHandle, Model, ModelRc, VecModel};

use super::artwork::artwork_jobs;
use super::fetch::fetch_page;
use super::filter::apply_filter;
use super::model::{to_item, BrowseCard};
use super::selected_tag;
use crate::adapter::SlintAdapter;
use crate::artwork::{self, ImageCache};
use crate::{AppWindow, PlaylistBrowseState, SearchPlaylistItem};

/// Fetch the next page (offset = PlaylistBrowseState.next-offset) and
/// append it. Wired to PlaylistBrowseActions::load-more; `genre_ids` is
/// the shared genre-filter selection. UI thread entry.
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
    let state = w.global::<PlaylistBrowseState>();
    if !state.get_has_more() || state.get_loading_more() || state.get_loading() {
        return;
    }
    // A non-empty search filters the loaded set client-side; pulling more
    // pages while filtering matches no UX (Tauri disables load-more too).
    if !state.get_search_query().is_empty() {
        return;
    }
    let offset = state.get_next_offset().max(0) as u32;
    let base_index = state.get_playlists().row_count();
    state.set_loading_more(true);
    let selected = selected_tag();

    handle.spawn(async move {
        match fetch_page(&runtime, &selected, genre_ids, offset).await {
            Ok((cards, has_more)) => {
                let jobs = artwork_jobs(&cards, base_index);
                let _ = weak.upgrade_in_event_loop(move |w| {
                    append_playlists(&w, cards, has_more);
                });
                artwork::spawn_loads(jobs, weak.clone(), image_cache.clone());
            }
            Err(e) => {
                log::error!("[qbz-slint] playlist-browse load-more failed: {e}");
                let _ = weak.upgrade_in_event_loop(|w| {
                    w.global::<PlaylistBrowseState>().set_loading_more(false);
                });
            }
        }
    });
}

/// Append a freshly-fetched page onto the loaded set, advancing the
/// offset and updating has-more. UI thread only.
fn append_playlists(window: &AppWindow, cards: Vec<BrowseCard>, has_more: bool) {
    let state = window.global::<PlaylistBrowseState>();
    let model = state.get_playlists();
    let mut combined: Vec<SearchPlaylistItem> = (0..model.row_count())
        .filter_map(|i| model.row_data(i))
        .collect();
    combined.extend(cards.iter().map(to_item));
    state.set_playlists(ModelRc::new(VecModel::from(combined)));
    state.set_next_offset(state.get_next_offset() + cards.len() as i32);
    state.set_has_more(has_more);
    state.set_loading_more(false);
    apply_filter(window);
}
