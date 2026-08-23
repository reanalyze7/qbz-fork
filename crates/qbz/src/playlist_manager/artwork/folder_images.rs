//! Folder custom images: local files (not http), read + decoded directly
//! rather than through the URL artwork pipeline.

use slint::{ComponentHandle, Model};

use crate::artwork;
use crate::folders::FolderFull;
use crate::{AppWindow, PlaylistManagerState};

use super::super::types::CACHE;

/// Decode the folder cards' custom images (local files) on a worker and
/// push them into the folder model. Folder custom images come from
/// `library.db`; the URL pipeline only handles http(s), so these are read
/// + decoded directly here.
pub fn load_folder_custom_images(weak: slint::Weak<AppWindow>, handle: &tokio::runtime::Handle) {
    let data = CACHE.lock().map(|c| c.clone()).unwrap_or_default();
    let with_images: Vec<(String, String)> = data
        .folders
        .iter()
        .filter(|f| f.icon_type == "custom")
        .filter_map(|f| f.custom_image_path.clone().map(|p| (f.id.clone(), p)))
        .collect();
    if with_images.is_empty() {
        return;
    }
    handle.spawn(async move {
        for (folder_id, path) in with_images {
            let path2 = path.clone();
            let decoded =
                tokio::task::spawn_blocking(move || decode_local_image(&path2, 160)).await;
            if let Ok(Some((pixels, w, h))) = decoded {
                let fid = folder_id.clone();
                let _ = weak.upgrade_in_event_loop(move |win| {
                    set_folder_image(&win, &fid, &pixels, w, h);
                });
            }
        }
    });
}

/// Read + decode a local image file to RGBA8, downscaled to `size`.
fn decode_local_image(path: &str, size: u32) -> Option<(Vec<u8>, u32, u32)> {
    let img = image::open(path).ok()?.thumbnail(size, size).to_rgba8();
    let (w, h) = img.dimensions();
    Some((img.into_raw(), w, h))
}

/// Look up a folder's full record (from the cache) for the editor.
pub fn folder_for_edit(folder_id: &str) -> Option<FolderFull> {
    CACHE
        .lock()
        .ok()?
        .folders
        .iter()
        .find(|f| f.id == folder_id)
        .cloned()
}

/// Decode a local image file and push it into the folder-editor preview
/// (FolderEditState.custom-image). Used when opening the editor on a
/// folder with an existing custom image, and after the user picks one.
pub fn load_editor_custom_image(weak: slint::Weak<AppWindow>, path: String) {
    std::thread::spawn(move || {
        if let Some((pixels, w, h)) = decode_local_image(&path, 160) {
            let _ = weak.upgrade_in_event_loop(move |win| {
                let image = artwork::pixels_to_image(&pixels, w, h);
                win.global::<crate::FolderEditState>().set_custom_image(image);
            });
        }
    });
}

fn set_folder_image(window: &AppWindow, folder_id: &str, pixels: &[u8], w: u32, h: u32) {
    let image = artwork::pixels_to_image(pixels, w, h);
    let model = window.global::<PlaylistManagerState>().get_folders();
    for i in 0..model.row_count() {
        if let Some(mut item) = model.row_data(i) {
            if item.id == folder_id {
                item.custom_image = image.clone();
                model.set_row_data(i, item);
            }
        }
    }
    // Mirror into the tree's folder rows.
    let tree = window.global::<PlaylistManagerState>().get_tree();
    for i in 0..tree.row_count() {
        if let Some(mut row) = tree.row_data(i) {
            if row.kind == "folder" && row.folder.id == folder_id {
                row.folder.custom_image = image.clone();
                tree.set_row_data(i, row);
            }
        }
    }
}
