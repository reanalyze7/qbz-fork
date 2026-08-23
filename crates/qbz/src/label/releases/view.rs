use slint::{ComponentHandle, Model, ModelRc, VecModel};

use super::derive::derive_releases;
use super::LabelData;
use crate::album_map::{to_item, AlbumCard};
use crate::artwork::{ArtworkJob, ArtworkTarget};
use crate::{AlbumCardItem, AppWindow, DiscoverSection, LabelState};

pub fn apply_label(window: &AppWindow, data: LabelData) {
    let items: Vec<AlbumCardItem> = data.albums.into_iter().map(to_item).collect();
    let state = window.global::<LabelState>();
    state.set_id(data.id.into());
    state.set_name(data.name.into());
    state.set_image_url(data.image_url.into());
    state.set_albums(ModelRc::new(VecModel::from(items)));
    state.set_total(data.total as i32);
    state.set_has_more(data.has_more);
    state.set_loading(false);
    derive_releases(window);
}

pub fn append_albums(window: &AppWindow, albums: Vec<AlbumCard>, total: usize, has_more: bool) {
    let state = window.global::<LabelState>();
    let model = state.get_albums();
    let mut combined: Vec<AlbumCardItem> = (0..model.row_count())
        .filter_map(|i| model.row_data(i))
        .collect();
    combined.extend(albums.into_iter().map(to_item));
    state.set_albums(ModelRc::new(VecModel::from(combined)));
    state.set_total(total as i32);
    state.set_has_more(has_more);
    state.set_load_more_loading(false);
    derive_releases(window);
}

/// Apply the decoded label header image. Runs on the Slint event loop.
pub fn apply_image(window: &AppWindow, pixels: &[u8], width: u32, height: u32) {
    let mut buffer = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(width, height);
    let dst = buffer.make_mut_bytes();
    if dst.len() != pixels.len() {
        return;
    }
    dst.copy_from_slice(pixels);
    window
        .global::<LabelState>()
        .set_image(slint::Image::from_rgba8(buffer));
}

pub fn reset_label(window: &AppWindow) {
    let state = window.global::<LabelState>();
    state.set_name("".into());
    state.set_image_url("".into());
    state.set_albums(ModelRc::new(VecModel::from(Vec::<AlbumCardItem>::new())));
    state.set_visible(ModelRc::new(VecModel::from(Vec::<AlbumCardItem>::new())));
    state.set_grouped(ModelRc::new(VecModel::from(Vec::<DiscoverSection>::new())));
    state.set_total(0);
    state.set_has_more(false);
    state.set_loading(true);
    state.set_load_more_loading(false);
    // Reset the toolbar to defaults for the fresh label.
    state.set_sort_by("newest".into());
    state.set_filter_hires(false);
    state.set_group_by_artist(false);
    state.set_search_query("".into());
    state.set_shown(0);
    state.set_hires_count(0);
}

/// Artwork jobs for the label album grid — same pipeline the
/// Discover cards use.
pub fn artwork_jobs(data: &LabelData) -> Vec<ArtworkJob> {
    data.albums
        .iter()
        .enumerate()
        .filter(|(_, a)| !a.artwork_url.is_empty())
        .map(|(i, a)| ArtworkJob {
            url: a.artwork_url.clone(),
            target: ArtworkTarget::LabelAlbum { index: i },
        })
        .collect()
}
