//! Derive the folder-detail subfolder search view, and paint resolved covers.

use slint::{ComponentHandle, Model, ModelRc, VecModel};

use crate::{AppWindow, FolderSubcardItem, LocalLibraryState};

/// Re-derive the rendered subfolder set (`-visible`) from the full set, filtered
/// by the subfolder name search. Mirrors `derive_folders`.
pub(crate) fn derive_folder_detail(window: &AppWindow) {
    let s = window.global::<LocalLibraryState>();
    let full = s.get_folder_detail_subfolders();
    let q = s.get_folder_detail_search().as_str().trim().to_lowercase();
    let rows: Vec<FolderSubcardItem> = (0..full.row_count())
        .filter_map(|i| full.row_data(i))
        .filter(|it| q.is_empty() || it.name.as_str().to_lowercase().contains(&q))
        .collect();
    s.set_folder_detail_subfolders_visible(ModelRc::new(VecModel::from(rows)));
}

/// Set the search filter for the subfolder cards and re-derive.
pub fn folder_detail_search(window: &AppWindow, query: &str) {
    window
        .global::<LocalLibraryState>()
        .set_folder_detail_search(query.into());
    derive_folder_detail(window);
}

/// Dual-set a resolved cover onto the full + visible subfolder sets by path.
pub fn set_folder_detail_subfolder_artwork(window: &AppWindow, path: &str, image: slint::Image) {
    let s = window.global::<LocalLibraryState>();
    let set_in = |m: &ModelRc<FolderSubcardItem>| {
        for i in 0..m.row_count() {
            if let Some(mut it) = m.row_data(i) {
                if it.path.as_str() == path {
                    it.artwork = image.clone();
                    m.set_row_data(i, it);
                    break;
                }
            }
        }
    };
    set_in(&s.get_folder_detail_subfolders());
    set_in(&s.get_folder_detail_subfolders_visible());
}
