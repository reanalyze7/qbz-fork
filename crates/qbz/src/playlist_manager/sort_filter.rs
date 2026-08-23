//! Sort comparators + search/visibility/folder filtering, over both the
//! Qobuz (`PmPlaylist`) and LOCAL (`PmLocalPlaylist`) row sets.

use crate::PmPlaylistItem;

use super::build::{local_playlist_item, playlist_item};
use super::types::{PmData, PmLocalPlaylist, PmPlaylist};

/// Order playlists by the active sort (mirrors `applySortToList`):
/// name (locale-ish), playcount desc, tracks (remote+local) desc, custom
/// (position asc); `recent` keeps the API order.
pub(super) fn sort_playlists(list: &mut [PmPlaylist], sort: &str) {
    match sort {
        "name" => list.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
        "playcount" => list.sort_by(|a, b| b.play_count.cmp(&a.play_count)),
        "tracks" => list.sort_by(|a, b| b.total_count().cmp(&a.total_count())),
        "custom" => list.sort_by(|a, b| a.position.cmp(&b.position)),
        // "recent" — keep insertion (API) order.
        _ => {}
    }
}

/// Display-stage union of a Qobuz playlist and a LOCAL (library.db)
/// playlist, so locals INTERLEAVE into the active sort instead of being
/// appended after the Qobuz set (B4). The u64-keyed mutators (custom-order
/// reorder, move-to-folder) still parse the model ids and skip `local:`
/// ones — the folder tables are Qobuz-keyed; favorite / hide route to the
/// local repo instead (B3, `toggle_local_favorite` / `toggle_local_hidden`).
///
/// Sort keys locals don't have:
/// - playcount: no local play stat — locals sort as ZERO, which under the
///   descending playcount sort puts them last (after any played Qobuz set);
/// - custom: positions are a Qobuz-side concept — locals sort as MAX, i.e.
///   after the positioned Qobuz set;
/// - recent: no recency signal — kept after the API-ordered Qobuz set.
/// Ties keep the pre-sort order (stable sort): API order for Qobuz rows,
/// name order among the locals.
pub(super) enum PmEntry<'a> {
    Qobuz(&'a PmPlaylist),
    Local(&'a PmLocalPlaylist),
}

impl PmEntry<'_> {
    fn name_lower(&self) -> String {
        match self {
            Self::Qobuz(p) => p.name.to_lowercase(),
            Self::Local(p) => p.name.to_lowercase(),
        }
    }

    fn total_count(&self) -> u32 {
        match self {
            Self::Qobuz(p) => p.total_count(),
            Self::Local(p) => p.track_count,
        }
    }

    fn play_count(&self) -> u32 {
        match self {
            Self::Qobuz(p) => p.play_count,
            Self::Local(_) => 0,
        }
    }

    fn position(&self) -> i64 {
        match self {
            Self::Qobuz(p) => p.position as i64,
            Self::Local(_) => i64::MAX,
        }
    }

    pub(super) fn item(&self) -> PmPlaylistItem {
        match self {
            Self::Qobuz(p) => playlist_item(p),
            Self::Local(p) => local_playlist_item(p),
        }
    }
}

/// `sort_playlists`, over the merged Qobuz + local display set (same
/// comparators; see `PmEntry` for the missing-stat rules).
pub(super) fn sort_entries(list: &mut [PmEntry], sort: &str) {
    match sort {
        "name" => list.sort_by_key(|e| e.name_lower()),
        "playcount" => list.sort_by(|a, b| b.play_count().cmp(&a.play_count())),
        "tracks" => list.sort_by(|a, b| b.total_count().cmp(&a.total_count())),
        "custom" => list.sort_by_key(|e| e.position()),
        // "recent" — Qobuz keeps API order, locals stay after it.
        _ => {}
    }
}

/// The LOCAL playlists that pass the toolbar filters, name-sorted (their
/// tie/no-stat order inside `sort_entries`). The visibility filter applies
/// to their own hidden flag (B3, `local_playlists.hidden`); folder
/// filtering N/A (root-only). `query` must already be lowercased.
pub(super) fn local_entries<'a>(data: &'a PmData, query: &str, filter: &str) -> Vec<&'a PmLocalPlaylist> {
    let mut locals: Vec<&PmLocalPlaylist> = data
        .locals
        .iter()
        .filter(|p| query.is_empty() || p.name.to_lowercase().contains(query))
        .filter(|p| match filter {
            "visible" => !p.is_hidden,
            "hidden" => p.is_hidden,
            _ => true,
        })
        .collect();
    locals.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    locals
}

/// Whether a playlist passes the search + visibility + folder filters.
/// `current_folder` is None for the flat / root view (we never enter a
/// folder in the Slint port — folder navigation is via the tree).
pub(super) fn passes(p: &PmPlaylist, query: &str, filter: &str, folder_mode: bool, view_mode: &str) -> bool {
    if !query.is_empty() && !p.name.to_lowercase().contains(query) {
        return false;
    }
    // Folder filter: in folder mode (non-tree), the grid/list shows ONLY
    // root playlists (folders own their members; opening a folder is the
    // tree's job in this port).
    if folder_mode && view_mode != "tree" && p.folder_id.is_some() {
        return false;
    }
    match filter {
        "visible" => !p.is_hidden,
        "hidden" => p.is_hidden,
        _ => true,
    }
}
