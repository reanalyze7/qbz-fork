//! My QBZ (Mixtapes/Collections + detail view) arms of `apply_artwork`.

use slint::{ComponentHandle, Model};

use crate::artwork::target::ArtworkTarget;
use crate::AppWindow;

pub(in crate::artwork) fn apply(window: &AppWindow, target: ArtworkTarget, image: &slint::Image) -> bool {
    match target {
        ArtworkTarget::MyQbzMixtapeCover { index, slot } => {
            let model = window.global::<crate::MyQbzState>().get_mixtapes();
            if let Some(mut item) = model.row_data(index) {
                crate::myqbz::set_mosaic_cover(&mut item, slot, image.clone());
                model.set_row_data(index, item);
            }
        }
        ArtworkTarget::MyQbzCollectionCover { index, slot } => {
            let model = window.global::<crate::MyQbzState>().get_collections();
            if let Some(mut item) = model.row_data(index) {
                crate::myqbz::set_mosaic_cover(&mut item, slot, image.clone());
                model.set_row_data(index, item);
            }
        }
        ArtworkTarget::MyQbzDetailRow { position } => {
            crate::myqbz_detail::set_row_artwork(window, position, image.clone());
        }
        ArtworkTarget::MyQbzDetailCover { slot } => {
            crate::myqbz_detail::set_hero_cover(window, slot, image.clone());
        }
        _ => return false,
    }
    true
}
