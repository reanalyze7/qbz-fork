//! The Qobuz-Playlists category-tag multi-select filter (client-side, no
//! network): toggle/clear a tag selection and re-render the filtered row.

use slint::{ComponentHandle, ModelRc, VecModel};

use crate::artwork::ArtworkJob;
use crate::{AppWindow, DiscoverState, HomeState};

use super::present::{playlist_artwork_jobs, playlist_to_item};
use super::{filter_playlists, PlaylistCardData, TAB_SECTIONS};

/// Re-push the active tab's Qobuz Playlists row filtered by `selected_tags`,
/// and return the artwork jobs for the (filtered) row. Shared by the toggle /
/// clear callbacks: the selection is already updated in the cache. For You has
/// no playlists row, so it returns no jobs.
fn rerender_playlists_filtered(window: &AppWindow) -> Vec<ArtworkJob> {
    let active = window.global::<DiscoverState>().get_active_tab().to_string();
    if active == "forYou" {
        return Vec::new();
    }
    let editor = active == "editorPicks";
    let hstate = window.global::<HomeState>();
    let (pls, jobs) = TAB_SECTIONS.with(|cell| {
        let cache = cell.borrow();
        let source = if editor {
            &cache.editor_playlists
        } else {
            &cache.home_playlists
        };
        let filtered: Vec<PlaylistCardData> = filter_playlists(source, &cache.selected_tags)
            .into_iter()
            .cloned()
            .collect();
        let jobs = playlist_artwork_jobs(&filtered);
        (
            filtered.iter().map(playlist_to_item).collect::<Vec<_>>(),
            jobs,
        )
    });
    hstate.set_playlists(ModelRc::new(VecModel::from(pls)));
    jobs
}

/// Toggle one category tag (by slug) in the Qobuz Playlists filter, re-filter
/// the cached row, and return the artwork jobs for the new (filtered) row. Also
/// updates the `playlist-tags[i].selected` flags + `playlist-tag-count` so the
/// dropdown reflects the selection.
pub fn toggle_playlist_tag(window: &AppWindow, slug: &str) -> Vec<ArtworkJob> {
    let count = TAB_SECTIONS.with(|cell| {
        let mut cache = cell.borrow_mut();
        if let Some(pos) = cache.selected_tags.iter().position(|s| s == slug) {
            cache.selected_tags.remove(pos);
        } else {
            cache.selected_tags.push(slug.to_string());
        }
        cache.selected_tags.len() as i32
    });
    sync_tag_selection(window, count);
    rerender_playlists_filtered(window)
}

/// Clear every selected category tag (show all playlists). Returns the artwork
/// jobs for the now-unfiltered row.
pub fn clear_playlist_tags(window: &AppWindow) -> Vec<ArtworkJob> {
    TAB_SECTIONS.with(|cell| cell.borrow_mut().selected_tags.clear());
    sync_tag_selection(window, 0);
    rerender_playlists_filtered(window)
}

/// Mirror the cached selection onto `HomeState.playlist-tags[i].selected` and
/// publish the selected count. Reads the selection from the cache so the two
/// never drift.
fn sync_tag_selection(window: &AppWindow, count: i32) {
    use slint::Model;
    // Snapshot the selection so the cache borrow is released before any Slint
    // model mutation (which can synchronously re-enter Rust closures).
    let selected: Vec<String> =
        TAB_SECTIONS.with(|cell| cell.borrow().selected_tags.clone());
    let model = window.global::<HomeState>().get_playlist_tags();
    for i in 0..model.row_count() {
        if let Some(mut item) = model.row_data(i) {
            let is_sel = selected.iter().any(|s| s.as_str() == item.slug.as_str());
            if item.selected != is_sel {
                item.selected = is_sel;
                model.set_row_data(i, item);
            }
        }
    }
    window.global::<HomeState>().set_playlist_tag_count(count);
}
