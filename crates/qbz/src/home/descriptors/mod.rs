//! Slice 5 — prefs-driven descriptor lists for Home / Editor's Picks, plus
//! the tab-switch entry point.

mod build;
mod renderable;

use qbz_app::settings::discover_prefs::{DiscoverPrefs, DiscoveryTab};
use slint::{ComponentHandle, ModelRc, VecModel};

use crate::artwork::ArtworkJob;
use crate::{AppWindow, DiscoverState, HomeState, SearchPlaylistItem, SectionDescriptor};

use build::{descriptors_for, discover_section_artwork_jobs};

use super::present::{playlist_artwork_jobs, playlist_to_item};
use super::{filter_playlists, PlaylistCardData, TAB_SECTIONS};

/// Build the Home + Editor's Picks descriptor lists from the cached section
/// data (called by the configurator controller after a mutation and at seed).
pub fn tab_descriptors(prefs: &DiscoverPrefs) -> (Vec<SectionDescriptor>, Vec<SectionDescriptor>) {
    TAB_SECTIONS.with(|cell| {
        let cache = cell.borrow();
        let home = descriptors_for(prefs, DiscoveryTab::Home, &cache.home);
        let editor = descriptors_for(prefs, DiscoveryTab::EditorPicks, &cache.editor);
        (home, editor)
    })
}

/// Re-render the active Home / Editor's Picks tab from the cached section data
/// (no network) after a configurator mutation: push the recomputed descriptor
/// lists + the active tab's Qobuz Playlists row, and return the descriptor
/// artwork jobs to re-fire for the active tab. For You is not handled here (its
/// data lives in ForYouState; the descriptor list alone drives it).
pub fn rerender_active_tab(window: &AppWindow, prefs: &DiscoverPrefs) -> Vec<ArtworkJob> {
    let active = window.global::<DiscoverState>().get_active_tab().to_string();
    if active == "forYou" {
        return Vec::new();
    }
    let editor = active == "editorPicks";
    let (home, editor_list) = tab_descriptors(prefs);
    let active_list = if editor { editor_list.clone() } else { home.clone() };

    let dstate = window.global::<DiscoverState>();
    dstate.set_home_sections(ModelRc::new(VecModel::from(home)));
    dstate.set_editor_sections(ModelRc::new(VecModel::from(editor_list)));

    // Re-push the active tab's Qobuz Playlists row (category-filtered) + build
    // the album-section artwork jobs from the same cached data (one borrow of
    // the cache). The playlist artwork jobs are built from the SAME filtered
    // slice, so their `idx` aligns with the pushed (filtered) row.
    let hstate = window.global::<HomeState>();
    let (pls, jobs) = TAB_SECTIONS.with(|cell| {
        let cache = cell.borrow();
        let (album_cache, pls) = if editor {
            (&cache.editor, &cache.editor_playlists)
        } else {
            (&cache.home, &cache.home_playlists)
        };
        let filtered: Vec<PlaylistCardData> = filter_playlists(pls, &cache.selected_tags)
            .into_iter()
            .cloned()
            .collect();
        let mut jobs = discover_section_artwork_jobs(&active_list, album_cache, editor);
        jobs.extend(playlist_artwork_jobs(&filtered));
        (
            filtered.iter().map(playlist_to_item).collect::<Vec<_>>(),
            jobs,
        )
    });
    hstate.set_playlists(ModelRc::new(VecModel::from(pls)));

    jobs
}

/// Switch the visible Discover tab ("home" | "editorPicks" | "forYou"). Writes
/// the active tab into BOTH HomeState (Slice-3 pill bindings) and DiscoverState
/// (the prefs-driven render loop + the configurator target — single source of
/// truth), then re-renders the active tab from the cached section data via the
/// descriptor lists. No re-fetch. For You renders from its own ForYouView /
/// ForYouState; the Home/Editor descriptor lists are pushed empty for it.
pub fn select_tab(window: &AppWindow, tab: &str) -> Vec<ArtworkJob> {
    window.global::<HomeState>().set_active_tab(tab.into());
    window.global::<DiscoverState>().set_active_tab(tab.into());

    if tab == "forYou" || tab == "recommendations" {
        // For You + Recommendations both render from their own dedicated state /
        // view; push the For You descriptor list + drive Home/Editor empty, and
        // clear the legacy HomeState models so nothing lingers underneath.
        let prefs = crate::discover_prefs::prefs_snapshot();
        crate::discover_prefs::push_descriptors(window, &prefs);
        let hstate = window.global::<HomeState>();
        hstate.set_playlists(ModelRc::new(VecModel::from(Vec::<SearchPlaylistItem>::new())));
        return Vec::new();
    }

    let prefs = crate::discover_prefs::prefs_snapshot();
    rerender_active_tab(window, &prefs)
}
