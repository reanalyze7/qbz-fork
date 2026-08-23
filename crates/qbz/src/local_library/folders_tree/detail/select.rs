//! Select a folder in the tree: load its detail pane (direct child tracks +
//! immediate subfolders for in-pane drill-down).

use slint::{ComponentHandle, Model, ModelRc, VecModel};

use crate::artwork::{ArtworkJob, ArtworkTarget, ImageCache};
use crate::{AppWindow, FolderSubcardItem, LocalLibraryState, TrackItem};

use crate::local_library::folders_tree::nodes::path_basename;
use crate::local_library::tracks::map::map_local_track;

use super::derive::derive_folder_detail;
use super::fetch::fetch_folder_detail;

/// Select a folder in the tree: load its detail pane — direct child tracks
/// plus immediate subfolders (for in-pane drill-down).
pub fn select_folder(
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    image_cache: ImageCache,
    path: String,
    segment: String,
) {
    let _ = weak.upgrade_in_event_loop({
        let path = path.clone();
        // Drill-in from a subfolder card passes an empty segment — derive the
        // display name from the path so the detail header is never blank.
        let segment = if segment.is_empty() {
            path_basename(&path)
        } else {
            segment.clone()
        };
        move |w| {
            let s = w.global::<LocalLibraryState>();
            s.set_folders_selected_path(path.clone().into());
            s.set_folders_selected_name(segment.clone().into());
            s.set_folder_detail_loading(true);
            // Reset the per-folder subfolder filter on navigation.
            s.set_folder_detail_search("".into());
        }
    });
    let path_for_fetch = path.clone();
    handle.spawn(async move {
        let (tracks, subfolders) =
            tokio::task::spawn_blocking(move || fetch_folder_detail(&path_for_fetch))
                .await
                .unwrap_or_default();
        let _ = weak.upgrade_in_event_loop(move |w| {
            let track_items: Vec<TrackItem> = tracks.into_iter().map(map_local_track).collect();
            // Subfolders become cover cards (1:1 with Tauri). The cover comes from
            // FolderTreeEntry::Folder.artwork (resolved async below).
            let cards: Vec<FolderSubcardItem> = subfolders
                .iter()
                .filter_map(|e| match e {
                    qbz_library::FolderTreeEntry::Folder {
                        path,
                        segment,
                        track_count_under,
                        artwork,
                    } => Some(FolderSubcardItem {
                        path: path.clone().into(),
                        name: segment.clone().into(),
                        track_count: *track_count_under as i32,
                        artwork: slint::Image::default(),
                        artwork_url: artwork.clone().unwrap_or_default().into(),
                    }),
                    _ => None,
                })
                .collect();
            // Recursive count = sum of subfolder counts + this folder's direct tracks.
            let recursive: i32 =
                cards.iter().map(|c| c.track_count).sum::<i32>() + track_items.len() as i32;
            let s = w.global::<LocalLibraryState>();
            s.set_folder_detail_tracks(ModelRc::new(VecModel::from(track_items)));
            s.set_folder_detail_track_count(recursive);
            s.set_folder_detail_subfolders(ModelRc::new(VecModel::from(cards)));
            s.set_folder_detail_loading(false);
            derive_folder_detail(&w);

            // Spawn cover artwork jobs over the full subfolder set.
            let full = s.get_folder_detail_subfolders();
            let mut jobs: Vec<ArtworkJob> = Vec::new();
            for i in 0..full.row_count() {
                if let Some(it) = full.row_data(i) {
                    let url = it.artwork_url.to_string();
                    if !url.is_empty() {
                        jobs.push(ArtworkJob {
                            target: ArtworkTarget::LocalFolderDetailCard { index: i },
                            url,
                        });
                    }
                }
            }
            if !jobs.is_empty() {
                crate::artwork::spawn_local_loads(jobs, w.as_weak(), image_cache.clone());
            }
        });
    });
}
