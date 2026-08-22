//! Sort helper for [`super::children`] — split out purely to keep
//! `children.rs` under the 130-line file cap.

use crate::FolderTreeEntry;

/// Sort: folders first, then tracks; alphabetical (case-insensitive)
/// within each group. Done in Rust because we already have all rows
/// in memory after the GROUP BY, and Rust's case-insensitive compare
/// is more obvious than COLLATE NOCASE on a CASE-derived column.
pub(super) fn sort_tree_entries(entries: &mut [FolderTreeEntry]) {
    entries.sort_by(|a, b| {
        let kind_rank = |e: &FolderTreeEntry| match e {
            FolderTreeEntry::Folder { .. } => 0,
            FolderTreeEntry::Track { .. } => 1,
        };
        let segment = |e: &FolderTreeEntry| match e {
            FolderTreeEntry::Folder { segment, .. } => segment.clone(),
            FolderTreeEntry::Track { segment, .. } => segment.clone(),
        };
        kind_rank(a)
            .cmp(&kind_rank(b))
            .then_with(|| segment(a).to_lowercase().cmp(&segment(b).to_lowercase()))
    });
}
