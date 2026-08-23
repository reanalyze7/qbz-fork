//! `navigate()` — the main "open this page" entry point.

use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use qbz_models::PlaylistTag;
use slint::{ComponentHandle, ModelRc, VecModel};

use super::artwork::artwork_jobs;
use super::fetch::fetch_page;
use super::filter::apply_filter;
use super::model::to_item;
use super::{selected_tag, SELECTED_TAG};
use crate::adapter::SlintAdapter;
use crate::artwork::{self, ImageCache};
use crate::{AppWindow, ContentView, NavState, PlaylistBrowseState, PlaylistTagItem, SearchPlaylistItem};

/// Open the Qobuz Playlists full-list page: reset the page state, switch
/// the view, fetch the tag bar + the first page concurrently, then fan
/// out artwork. `genre_ids` is the shared genre-filter selection (None =
/// no filter). `reset_tag` picks the tag semantics: true on a fresh open
/// from the rail's "View all" (back to All), false on genre-filter /
/// history re-navigations (the selected tab survives).
pub fn navigate(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: ImageCache,
    genre_ids: Option<Vec<u64>>,
    reset_tag: bool,
) {
    if reset_tag {
        if let Ok(mut s) = SELECTED_TAG.lock() {
            s.clear();
        }
    }
    let selected = selected_tag();
    handle.spawn(async move {
        // Reset the page state and switch the view on the UI thread. The
        // search query clears on a fresh navigation; the view mode and the
        // selected tag persist (the tag was cleared above when this is a
        // fresh open from the rail).
        {
            let selected = selected.clone();
            let _ = weak.upgrade_in_event_loop(move |w| {
                let state = w.global::<PlaylistBrowseState>();
                state.set_title(qbz_i18n::t("Qobuz Playlists").into());
                state.set_selected_tag(selected.into());
                state.set_next_offset(0);
                state.set_search_query("".into());
                state.set_playlists(ModelRc::new(VecModel::from(
                    Vec::<SearchPlaylistItem>::new(),
                )));
                state.set_visible(ModelRc::new(VecModel::from(
                    Vec::<SearchPlaylistItem>::new(),
                )));
                state.set_loading(true);
                state.set_loading_more(false);
                state.set_has_more(true);
                w.global::<NavState>().set_view(ContentView::PlaylistBrowse);
            });
        }

        let (tags_res, page_res) = futures_util::join!(
            runtime.core().get_playlist_tags(),
            fetch_page(&runtime, &selected, genre_ids, 0)
        );

        // A tag-bar failure is non-fatal: the page still lists playlists.
        let tags: Vec<PlaylistTag> = match tags_res {
            Ok(tags) => tags,
            Err(e) => {
                log::warn!("[qbz-slint] playlist-browse tags load failed: {e}");
                Vec::new()
            }
        };

        match page_res {
            Ok((cards, has_more)) => {
                let jobs = artwork_jobs(&cards, 0);
                let _ = weak.upgrade_in_event_loop(move |w| {
                    let items: Vec<SearchPlaylistItem> = cards.iter().map(to_item).collect();
                    let fetched = items.len() as i32;
                    let state = w.global::<PlaylistBrowseState>();
                    state.set_tags(ModelRc::new(VecModel::from(
                        tags.into_iter()
                            .map(|t| {
                                let is_selected = t.slug == selected;
                                PlaylistTagItem {
                                    slug: t.slug.into(),
                                    name: t.name.into(),
                                    selected: is_selected,
                                }
                            })
                            .collect::<Vec<_>>(),
                    )));
                    state.set_playlists(ModelRc::new(VecModel::from(items)));
                    state.set_next_offset(fetched);
                    state.set_has_more(has_more);
                    state.set_loading(false);
                    apply_filter(&w);
                });
                artwork::spawn_loads(jobs, weak.clone(), image_cache.clone());
            }
            Err(e) => {
                log::error!("[qbz-slint] playlist-browse load failed: {e}");
                let _ = weak.upgrade_in_event_loop(|w| {
                    let state = w.global::<PlaylistBrowseState>();
                    state.set_loading(false);
                    state.set_has_more(false);
                });
            }
        }
    });
}
