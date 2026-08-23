use slint::{ComponentHandle, Model, ModelRc, VecModel};

use crate::{AppWindow, FavoriteLabelItem, FavoritesState};

/// Re-derive the rendered Labels grid (`labels-visible`) from the full
/// `labels` set and the search query. An empty query shares the full
/// model so artwork keeps updating in place; a query forks a filtered
/// clone (name-only substring match, mirrors Tauri's filteredLabels).
pub fn derive_labels(window: &AppWindow) {
    let state = window.global::<FavoritesState>();
    let query_owned = state.get_labels_search().to_lowercase();
    let query = query_owned.trim();
    let all = state.get_labels();
    if query.is_empty() {
        state.set_labels_visible(all);
        return;
    }
    let filtered: Vec<FavoriteLabelItem> = (0..all.row_count())
        .filter_map(|i| all.row_data(i))
        .filter(|l| l.name.to_lowercase().contains(query))
        .collect();
    state.set_labels_visible(ModelRc::new(VecModel::from(filtered)));
}
