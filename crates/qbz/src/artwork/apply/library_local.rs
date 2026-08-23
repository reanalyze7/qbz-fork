//! Library-all + Local Library arms of `apply_artwork`.

use slint::{ComponentHandle, Model};

use crate::artwork::target::ArtworkTarget;
use crate::AppWindow;

pub(in crate::artwork) fn apply(window: &AppWindow, target: ArtworkTarget, image: &slint::Image) -> bool {
    match target {
        ArtworkTarget::BlacklistAlbum { idx } => {
            let model = window.global::<crate::BlacklistState>().get_album_items();
            if let Some(mut item) = model.row_data(idx) {
                item.cover = image.clone();
                model.set_row_data(idx, item);
            }
        }
        ArtworkTarget::LibraryAllCover { index } => {
            let model = window.global::<crate::LibraryAllState>().get_items_visible();
            if let Some(mut item) = model.row_data(index) {
                item.image = image.clone();
                model.set_row_data(index, item);
            }
        }
        ArtworkTarget::LocalAlbumById { id, gen } => {
            // The job is done either way — free its in-flight slot so the
            // window dispatcher can re-request it after an eviction.
            crate::local_library::album_artwork_job_done(&id);
            // Drop the cover if a reload superseded the set it belongs to.
            if !crate::local_library::albums_gen_current(gen) {
                return true;
            }
            // Set by id onto the full set + visible + grouped sections.
            crate::local_library::set_local_album_artwork(window, &id, image.clone());
        }
        ArtworkTarget::LocalFolderCard { index } => {
            let model = window.global::<crate::LocalLibraryState>().get_folders();
            if let Some(item) = model.row_data(index) {
                // Dual-set by id onto the full set + visible + grouped sections.
                let id = item.id.to_string();
                crate::local_library::set_local_folder_artwork(window, &id, image.clone());
            }
        }
        ArtworkTarget::LocalFolderDetailCard { index } => {
            let model = window
                .global::<crate::LocalLibraryState>()
                .get_folder_detail_subfolders();
            if let Some(item) = model.row_data(index) {
                let path = item.path.to_string();
                crate::local_library::set_folder_detail_subfolder_artwork(
                    window,
                    &path,
                    image.clone(),
                );
            }
        }
        ArtworkTarget::LocalArtistAlbumCard { index } => {
            let model = window
                .global::<crate::LocalLibraryState>()
                .get_artists_selected_albums();
            if let Some(mut item) = model.row_data(index) {
                item.artwork = image.clone();
                model.set_row_data(index, item);
            }
        }
        ArtworkTarget::LocalArtistRowImage { index, gen } => {
            // Drop a portrait whose artists list was superseded by a reload.
            if crate::local_library::artists_img_gen_current() == gen {
                let s = window.global::<crate::LocalLibraryState>();
                if let Some(item) = s.get_artists().row_data(index) {
                    let name = item.name.to_string();
                    crate::local_library::set_artist_row_image(window, &name, image.clone());
                }
            }
        }
        ArtworkTarget::LocalAlbumViewCover => {
            window.global::<crate::LocalAlbumState>().set_cover(image.clone());
        }
        _ => return false,
    }
    true
}
