//! Artwork propagation: the artwork pipeline writes a decoded cover into the
//! SOURCE model (`albums` / `tracks`) by index, but the views render the
//! derived `albums-visible` / `albums-grouped` / `tracks-visible` models,
//! which are independent clones whenever a sort / group / search is active.
//! So a late-arriving cover never reached the rendered card (it stayed grey
//! until a re-derive). Propagate it into the rendered model(s) by id too.

use slint::{ComponentHandle, Model, ModelRc};

use crate::{AlbumCardItem, AppWindow, FavoritesState};

pub(crate) fn set_artwork_in_albums(model: &ModelRc<AlbumCardItem>, id: &str, image: &slint::Image) {
    for i in 0..model.row_count() {
        if let Some(mut item) = model.row_data(i) {
            if item.id.as_str() == id {
                item.artwork = image.clone();
                model.set_row_data(i, item);
                break;
            }
        }
    }
}

/// Set a freshly-decoded album cover (by id) on every favorites album model:
/// the full `albums` set + the flat `albums-visible` + every `albums-grouped`
/// section (mirrors `local_library::set_local_album_artwork`).
pub fn set_album_artwork(window: &AppWindow, id: &str, image: slint::Image) {
    let st = window.global::<FavoritesState>();
    set_artwork_in_albums(&st.get_albums(), id, &image);
    set_artwork_in_albums(&st.get_albums_visible(), id, &image);
    let grouped = st.get_albums_grouped();
    for s in 0..grouped.row_count() {
        if let Some(section) = grouped.row_data(s) {
            set_artwork_in_albums(&section.albums, id, &image);
        }
    }
}

/// Set a freshly-decoded artist photo (by id) on the rendered favorites
/// artist models (flat `artists-visible` + every `artists-grouped` section,
/// which backs both the grouped grid and the sidepanel list). Without this
/// the photo only lands on the source `artists` model and the grouped/
/// sidepanel views show it only after a re-derive (revisit).
pub fn set_artist_image(window: &AppWindow, id: &str, image: slint::Image) {
    let st = window.global::<FavoritesState>();
    let vis = st.get_artists_visible();
    for i in 0..vis.row_count() {
        if let Some(mut item) = vis.row_data(i) {
            if item.id.as_str() == id {
                item.image = image.clone();
                vis.set_row_data(i, item);
                break;
            }
        }
    }
    let grouped = st.get_artists_grouped();
    for s in 0..grouped.row_count() {
        if let Some(section) = grouped.row_data(s) {
            let arts = section.artists;
            for i in 0..arts.row_count() {
                if let Some(mut item) = arts.row_data(i) {
                    if item.id.as_str() == id {
                        item.image = image.clone();
                        arts.set_row_data(i, item);
                        break;
                    }
                }
            }
        }
    }
}

/// Set a freshly-decoded collage cover (by id + slot) on the rendered
/// favorites playlists model (`playlists-visible`), which is a clone of the
/// active sub-tab source whenever a search filter is active.
pub fn set_playlist_cover(window: &AppWindow, id: &str, slot: usize, image: slint::Image) {
    let model = window.global::<FavoritesState>().get_playlists_visible();
    for i in 0..model.row_count() {
        if let Some(mut item) = model.row_data(i) {
            if item.id.as_str() == id {
                match slot {
                    0 => item.cover1 = image,
                    1 => item.cover2 = image,
                    2 => item.cover3 = image,
                    _ => item.cover4 = image,
                }
                model.set_row_data(i, item);
                break;
            }
        }
    }
}

/// Same for the rendered favorites tracks model (`tracks-visible`).
pub fn set_track_artwork(window: &AppWindow, id: &str, image: slint::Image) {
    let model = window.global::<FavoritesState>().get_tracks_visible();
    for i in 0..model.row_count() {
        if let Some(mut item) = model.row_data(i) {
            if item.id.as_str() == id {
                item.artwork = image.clone();
                model.set_row_data(i, item);
                break;
            }
        }
    }
}
