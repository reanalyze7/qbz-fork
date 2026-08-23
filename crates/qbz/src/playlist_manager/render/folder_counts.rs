//! Folder member counts (regardless of search/visibility, like Tauri) —
//! shared by `rebuild` and `search_menu_folders`.

use std::collections::HashMap;

use crate::PmFolderItem;

use super::super::build::folder_item;
use super::super::types::PmData;

pub(super) fn folder_counts(data: &PmData) -> HashMap<String, usize> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for p in &data.playlists {
        if let Some(fid) = &p.folder_id {
            *counts.entry(fid.clone()).or_insert(0) += 1;
        }
    }
    counts
}

pub(super) fn folder_items(data: &PmData, counts: &HashMap<String, usize>) -> Vec<PmFolderItem> {
    data.folders
        .iter()
        .map(|f| folder_item(f, counts.get(&f.id).copied().unwrap_or(0)))
        .collect()
}
