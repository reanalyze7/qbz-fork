use slint::{ComponentHandle, Model, ModelRc, VecModel};

use crate::artist::cache::{FULL_RELEASE_SECTIONS, LOADED_PAGES, MAX_INDEX_PAGES};
use crate::{AlbumCardItem, AppWindow, ArtistReleaseSection, ArtistState};

/// Re-sort one release bucket in place (operates on the LIVE model so loaded
/// artwork is preserved) and persist the choice. `sort` = default | newest |
/// oldest | title-asc | title-desc.
pub fn resort_section(window: &AppWindow, release_type: &str, sort: &str) {
    crate::artist_prefs::set_sort(release_type, sort);
    let model = window.global::<ArtistState>().get_release_sections();
    for i in 0..model.row_count() {
        let Some(row) = model.row_data(i) else { continue };
        if row.release_type.as_str() != release_type {
            continue;
        }
        let mut albums: Vec<AlbumCardItem> = row.albums.iter().collect();
        crate::album_map::sort_album_items(&mut albums, sort);
        let new_row = ArtistReleaseSection {
            albums: ModelRc::new(VecModel::from(albums)),
            sort_by: sort.into(),
            ..row
        };
        model.set_row_data(i, new_row);
        break;
    }
    // Keep the FULL cache (in-page search source) in the same order.
    FULL_RELEASE_SECTIONS.with(|cell| {
        for s in cell.borrow_mut().iter_mut() {
            if s.release_type.as_str() == release_type {
                let mut albums: Vec<AlbumCardItem> = s.albums.iter().collect();
                crate::album_map::sort_album_items(&mut albums, sort);
                s.albums = ModelRc::new(VecModel::from(albums));
                s.sort_by = sort.into();
                break;
            }
        }
    });
}

/// Current loaded item count for a bucket — the offset for the next page.
pub fn section_loaded_count(window: &AppWindow, release_type: &str) -> usize {
    let model = window.global::<ArtistState>().get_release_sections();
    for i in 0..model.row_count() {
        if let Some(row) = model.row_data(i) {
            if row.release_type.as_str() == release_type {
                return row.albums.row_count();
            }
        }
    }
    0
}

/// Whether a bucket may still load another page on the index (cap = 4).
pub fn section_can_load_more(release_type: &str) -> bool {
    LOADED_PAGES.with(|c| c.borrow().get(release_type).copied().unwrap_or(1)) < MAX_INDEX_PAGES
}
