use slint::{ComponentHandle, ModelRc, VecModel};

use crate::favorites::derive::derive_labels;
use crate::favorites::mapping::LabelCard;
use crate::{AppWindow, FavoriteLabelItem, FavoritesState};

pub(crate) fn apply_labels(window: &AppWindow, items: Vec<LabelCard>, total: usize) {
    let state = window.global::<FavoritesState>();
    let rows: Vec<FavoriteLabelItem> = items
        .into_iter()
        .map(|l| FavoriteLabelItem {
            id: l.id.into(),
            name: l.name.into(),
            albums_line: l.albums_line.into(),
            image_url: l.image_url.into(),
            image: slint::Image::default(),
        })
        .collect();
    // `labels` is the full set the artwork pipeline targets;
    // `labels-visible` (what the grid renders) shares it until a
    // search filter forks it, so artwork stays live.
    let model = ModelRc::new(VecModel::from(rows));
    state.set_labels(model);
    state.set_labels_total(total as i32);
    state.set_labels_search("".into());
    derive_labels(window);
}
