//! Paint decoded covers onto the rendered Albums/Folders models.

use slint::{ComponentHandle, Model, ModelRc};

use crate::{AlbumCardItem, AppWindow, LocalLibraryState};

/// Set a freshly-decoded local-album cover (by id) on every rendered model:
/// the full `albums` set + `albums-visible` + each `albums-grouped` section.
pub fn set_local_album_artwork(window: &AppWindow, id: &str, image: slint::Image) {
    let s = window.global::<LocalLibraryState>();
    let set_in = |m: &ModelRc<AlbumCardItem>| {
        for i in 0..m.row_count() {
            if let Some(mut it) = m.row_data(i) {
                if it.id.as_str() == id {
                    it.artwork = image.clone();
                    m.set_row_data(i, it);
                    break;
                }
            }
        }
    };
    set_in(&s.get_albums());
    set_in(&s.get_albums_visible());
    let grouped = s.get_albums_grouped();
    for gi in 0..grouped.row_count() {
        if let Some(sec) = grouped.row_data(gi) {
            set_in(&sec.albums);
        }
    }
}

/// Same, for the Folders tab: full `folders` set + `folders-visible` + each
/// `folders-grouped` section. Without this the cover only lands on the source
/// `folders` model and the rendered (visible/grouped) views miss it on first
/// load (the same bug the Albums/Artists tabs had).
pub fn set_local_folder_artwork(window: &AppWindow, id: &str, image: slint::Image) {
    let s = window.global::<LocalLibraryState>();
    let set_in = |m: &ModelRc<AlbumCardItem>| {
        for i in 0..m.row_count() {
            if let Some(mut it) = m.row_data(i) {
                if it.id.as_str() == id {
                    it.artwork = image.clone();
                    m.set_row_data(i, it);
                    break;
                }
            }
        }
    };
    set_in(&s.get_folders());
    set_in(&s.get_folders_visible());
    let grouped = s.get_folders_grouped();
    for gi in 0..grouped.row_count() {
        if let Some(sec) = grouped.row_data(gi) {
            set_in(&sec.albums);
        }
    }
}
