use slint::{ComponentHandle, Model, ModelRc, VecModel};

use crate::album_map::{to_item, AlbumCard};
use crate::favorites::derive::derive_albums;
use crate::{AlbumCardItem, AppWindow, FavoritesState};

pub(crate) fn apply_albums(window: &AppWindow, items: Vec<AlbumCard>, total: usize) {
    let state = window.global::<FavoritesState>();
    // Everything in the Albums tab is a favorite -> filled heart.
    // (Blocked albums are already dropped at the FavData source;
    // artwork delivery is id-keyed, so no index alignment to keep.)
    let cards: Vec<AlbumCardItem> = items
        .into_iter()
        .map(|c| {
            let mut it = to_item(c);
            it.is_favorite = true;
            it
        })
        .collect();
    // `albums` is the full set (artwork target); `albums-visible`
    // (what the grid/list renders) shares it until a search/sort
    // forks it, so artwork stays live.
    let model = ModelRc::new(VecModel::from(cards));
    let n = model.row_count() as i32;
    state.set_albums(model.clone());
    state.set_albums_visible(model);
    state.set_albums_total(total as i32);
    state.set_albums_shown(n);
    state.set_albums_search("".into());
    // Apply the (persisted) sort + group to the freshly loaded set.
    derive_albums(window);
}
