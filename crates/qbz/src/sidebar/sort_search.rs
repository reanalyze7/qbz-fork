//! Active sort/search state + the pure playlist ordering.

use slint::ComponentHandle;

use crate::{AppWindow, SidebarState};

use super::rebuild::rebuild;
use super::{SidebarPlaylist, SEARCH, SORT};

/// Update the active sort option and re-render (matches Tauri's five
/// options). Unknown values fall back to "name".
pub fn set_sort(window: &AppWindow, option: &str) {
    let opt = match option {
        "name" | "recent" | "tracks" | "playcount" | "custom" => option,
        _ => "name",
    };
    if let Ok(mut s) = SORT.lock() {
        *s = opt.to_string();
    }
    window.global::<SidebarState>().set_sort_option(opt.into());
    rebuild(window);
}

/// Update the playlist-name search filter and re-render. An empty query
/// shows everything.
pub fn set_search(window: &AppWindow, query: &str) {
    if let Ok(mut q) = SEARCH.lock() {
        *q = query.trim().to_lowercase();
    }
    rebuild(window);
}

/// Order playlists by the active sort option, mirroring Tauri's
/// comparators. `recent` keeps reverse insertion order (most-recently
/// added first); `playcount` has no per-playlist count source here, so it
/// stays stable like Tauri does when `play_count` is absent (0).
pub(super) fn sort_playlists(playlists: &[SidebarPlaylist]) -> Vec<SidebarPlaylist> {
    let sort = SORT.lock().map(|s| s.clone()).unwrap_or_else(|_| "name".into());
    let mut out: Vec<SidebarPlaylist> = playlists.to_vec();
    match sort.as_str() {
        "name" => out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
        "recent" => out.reverse(),
        "tracks" => out.sort_by(|a, b| b.tracks_count.cmp(&a.tracks_count)),
        "playcount" => { /* no play_count source — stable, like Tauri's absent field */ }
        "custom" => out.sort_by(|a, b| a.position.cmp(&b.position)),
        _ => out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
    }
    out
}
