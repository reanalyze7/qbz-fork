use slint::{ComponentHandle, ModelRc, VecModel};

use crate::favorites::derive::derive_artists;
use crate::favorites::mapping::ArtistCard;
use crate::{AppWindow, FavoriteArtistItem, FavoritesState};

pub(crate) fn apply_artists(window: &AppWindow, items: Vec<ArtistCard>, total: usize) {
    let state = window.global::<FavoritesState>();
    let cards: Vec<FavoriteArtistItem> = items
        .into_iter()
        .map(|a| FavoriteArtistItem {
            is_pinned: crate::pinned::is_pinned("artist", &a.id),
            id: a.id.into(),
            name: a.name.into(),
            image_url: a.image_url.into(),
            image: slint::Image::default(),
        })
        .collect();
    let model = ModelRc::new(VecModel::from(cards));
    state.set_artists(model.clone());
    state.set_artists_visible(model);
    state.set_artists_total(total as i32);
    state.set_artists_search("".into());
    derive_artists(window);
}
