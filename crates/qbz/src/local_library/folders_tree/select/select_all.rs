//! "Select all" toggle over every track under the registered root folders.

use slint::ComponentHandle;

use crate::AppWindow;

use crate::local_library::folders_tree::nodes::{apply_tree, collect_tree};

use super::state::tree_selected;

/// Toggle "select all": if every track under the roots is already selected,
/// clear the selection; otherwise select them all. Two-way (the bulk button
/// flips select-all / un-select-all).
pub fn tree_select_all(weak: slint::Weak<AppWindow>, handle: tokio::runtime::Handle) {
    handle.spawn(async move {
        // Roots = the registered library folders.
        let paths = tokio::task::spawn_blocking(|| {
            crate::library_db::with_db(|db| db.get_folders()).unwrap_or_default()
        })
        .await
        .unwrap_or_default();
        let all = tokio::task::spawn_blocking(move || {
            let mut acc: Vec<qbz_library::LocalTrack> = Vec::new();
            for p in paths {
                let mut t =
                    crate::library_db::with_db(|db| db.list_folder_tracks_recursive(&p, false))
                        .unwrap_or_default();
                acc.append(&mut t);
            }
            acc
        })
        .await
        .unwrap_or_default();
        {
            let mut sel = tree_selected();
            let all_selected =
                !all.is_empty() && all.iter().all(|t| sel.contains_key(&t.file_path));
            if all_selected {
                // Un-select everything under the roots.
                for t in &all {
                    sel.remove(&t.file_path);
                }
            } else {
                for t in all {
                    sel.insert(t.file_path.clone(), t);
                }
            }
        }
        let _ = weak.upgrade_in_event_loop(|w| {
            let s = w.global::<crate::LocalLibraryState>();
            let nodes = collect_tree(&s);
            apply_tree(&s, nodes);
        });
    });
}
