//! Flatten folders + their (expanded) playlists + root playlists into the
//! tree model. Auto-expands all folders the first time the tree opens.

use crate::{PmFolderItem, PmPlaylistItem, PmTreeRow};

use super::super::build::{folder_item, playlist_item};
use super::super::sort_filter::{local_entries, sort_entries, sort_playlists, PmEntry};
use super::super::types::{PmData, PmPlaylist, EXPANDED, TREE_INIT};

pub(super) fn build_tree(data: &PmData, query: &str, filter: &str, sort: &str) -> Vec<PmTreeRow> {
    // Auto-expand on first tree open (Tauri's treeInitialized).
    {
        let mut init = TREE_INIT.lock().unwrap_or_else(|e| e.into_inner());
        if !*init {
            if let Ok(mut exp) = EXPANDED.lock() {
                for f in &data.folders {
                    exp.insert(f.id.clone());
                }
            }
            *init = true;
        }
    }
    let expanded = EXPANDED.lock().map(|e| e.clone()).unwrap_or_default();
    let searching = !query.is_empty();
    let offline = crate::offline_mode::engine().is_offline();

    let matches = |p: &PmPlaylist| -> bool {
        // D11.b: offline only the MIXED and snapshot-available (B8)
        // playlists stay.
        if offline && p.local_count == 0 && !p.offline_available {
            return false;
        }
        if searching && !p.name.to_lowercase().contains(query) {
            return false;
        }
        match filter {
            "visible" => !p.is_hidden,
            "hidden" => p.is_hidden,
            _ => true,
        }
    };

    let mut rows: Vec<PmTreeRow> = Vec::new();
    for f in &data.folders {
        let mut members: Vec<PmPlaylist> = data
            .playlists
            .iter()
            .filter(|p| p.folder_id.as_deref() == Some(f.id.as_str()))
            .filter(|p| matches(p))
            .cloned()
            .collect();
        // While searching — and offline, where the D11.b filter may empty a
        // folder — skip folders with no visible members.
        if (searching || offline) && members.is_empty() {
            continue;
        }
        sort_playlists(&mut members, sort);
        let is_exp = searching || expanded.contains(&f.id);
        rows.push(PmTreeRow {
            kind: "folder".into(),
            expanded: is_exp,
            folder: folder_item(f, members.len()),
            playlist: PmPlaylistItem::default(),
            indent: false,
        });
        if is_exp {
            for p in &members {
                rows.push(PmTreeRow {
                    kind: "playlist".into(),
                    expanded: false,
                    folder: PmFolderItem::default(),
                    playlist: playlist_item(p),
                    indent: true,
                });
            }
        }
    }
    // Root playlists (no folder), with the LOCAL playlists (never in
    // folders) interleaved into the SAME sort (B4) — see `PmEntry` for the
    // missing-stat sort rules.
    let root: Vec<PmPlaylist> = data
        .playlists
        .iter()
        .filter(|p| p.folder_id.is_none())
        .filter(|p| matches(p))
        .cloned()
        .collect();
    let mut entries: Vec<PmEntry> = root.iter().map(PmEntry::Qobuz).collect();
    entries.extend(local_entries(data, query, filter).into_iter().map(PmEntry::Local));
    sort_entries(&mut entries, sort);
    for e in &entries {
        rows.push(PmTreeRow {
            kind: "playlist".into(),
            expanded: false,
            folder: PmFolderItem::default(),
            playlist: e.item(),
            indent: false,
        });
    }
    rows
}
