//! Per-folder (recursive) and per-track selection toggles.

use slint::ComponentHandle;

use crate::AppWindow;

use crate::local_library::folders_tree::nodes::{apply_tree, collect_tree};

use super::state::tree_selected;

/// Toggle every track under a folder (recursive). 'all' → deselect; else select.
pub fn toggle_tree_folder_select(
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    path: String,
) {
    handle.spawn(async move {
        let p = path.clone();
        let tracks = tokio::task::spawn_blocking(move || {
            crate::library_db::with_db(|db| db.list_folder_tracks_recursive(&p, false))
                .unwrap_or_default()
        })
        .await
        .unwrap_or_default();
        if tracks.is_empty() {
            return;
        }
        let all_selected = {
            let sel = tree_selected();
            tracks.iter().all(|t| sel.contains_key(&t.file_path))
        };
        {
            let mut sel = tree_selected();
            if all_selected {
                for t in &tracks {
                    sel.remove(&t.file_path);
                }
            } else {
                for t in tracks {
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

/// Toggle a single track row by path.
pub fn toggle_tree_track_select(
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    path: String,
) {
    handle.spawn(async move {
        let was_selected = tree_selected().contains_key(&path);
        if was_selected {
            tree_selected().remove(&path);
        } else {
            // Resolve the track record from its parent folder listing.
            let parent = std::path::Path::new(&path)
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            let p = parent.clone();
            let tracks = tokio::task::spawn_blocking(move || {
                crate::library_db::with_db(|db| db.list_folder_tracks(&p, false)).unwrap_or_default()
            })
            .await
            .unwrap_or_default();
            if let Some(t) = tracks.into_iter().find(|t| t.file_path == path) {
                tree_selected().insert(path.clone(), t);
            }
        }
        let _ = weak.upgrade_in_event_loop(|w| {
            let s = w.global::<crate::LocalLibraryState>();
            let nodes = collect_tree(&s);
            apply_tree(&s, nodes);
        });
    });
}
