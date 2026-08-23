use std::collections::HashMap;

use slint::{ComponentHandle, Model, ModelRc, VecModel};

use crate::album_map::sort_album_items;
use crate::{AlbumCardItem, AppWindow, DiscoverSection, LabelState};

/// Re-derive the releases sub-view's rendered list (`visible` / `grouped`)
/// from the full loaded catalog + the toolbar state (sort / Hi-Res filter /
/// search / group-by-artist). Mirrors Tauri's client-side `$derived`
/// processing. Search is a local filter over the loaded catalog. Artwork
/// stays live in the common case because the no-filter, default-sort path
/// shares the `albums` model (the DiscoverBrowse pattern).
pub fn derive_releases(window: &AppWindow) {
    let state = window.global::<LabelState>();
    let albums = state.get_albums();
    let count = albums.row_count();
    let full: Vec<AlbumCardItem> = (0..count).filter_map(|i| albums.row_data(i)).collect();

    let sort = state.get_sort_by().to_string();
    let hires = state.get_filter_hires();
    let group = state.get_group_by_artist();
    let query_owned = state.get_search_query().to_lowercase();
    let query = query_owned.trim();

    let hires_count = full
        .iter()
        .filter(|a| a.quality_tier.as_str() == "hires")
        .count();

    // Fast path: default order, no filter/search, flat → render the live
    // `albums` model directly so artwork keeps updating in place.
    if !hires && query.is_empty() && sort == "newest" && !group {
        state.set_visible(albums.clone());
        state.set_grouped(ModelRc::new(VecModel::from(Vec::<DiscoverSection>::new())));
        state.set_shown(full.len() as i32);
        state.set_hires_count(hires_count as i32);
        return;
    }

    let mut filtered: Vec<AlbumCardItem> = full
        .into_iter()
        .filter(|a| {
            (!hires || a.quality_tier.as_str() == "hires")
                && (query.is_empty()
                    || a.title.to_lowercase().contains(query)
                    || a.artist.to_lowercase().contains(query))
        })
        .collect();
    sort_album_items(&mut filtered, &sort);
    let shown = filtered.len();

    if group {
        // One section per artist, in first-appearance (sorted) order.
        let mut sections: Vec<DiscoverSection> = Vec::new();
        let mut buckets: Vec<Vec<AlbumCardItem>> = Vec::new();
        let mut index: HashMap<String, usize> = HashMap::new();
        for item in filtered {
            let key = item.artist.to_string();
            let idx = *index.entry(key.clone()).or_insert_with(|| {
                buckets.push(Vec::new());
                sections.push(DiscoverSection {
                    title: if key.is_empty() {
                        qbz_i18n::t("Unknown").into()
                    } else {
                        key.clone().into()
                    },
                    endpoint: "".into(),
                    albums: ModelRc::new(VecModel::from(Vec::<AlbumCardItem>::new())),
                });
                buckets.len() - 1
            });
            buckets[idx].push(item);
        }
        for (i, bucket) in buckets.into_iter().enumerate() {
            sections[i].albums = ModelRc::new(VecModel::from(bucket));
        }
        state.set_grouped(ModelRc::new(VecModel::from(sections)));
        state.set_visible(ModelRc::new(VecModel::from(Vec::<AlbumCardItem>::new())));
    } else {
        state.set_visible(ModelRc::new(VecModel::from(filtered)));
        state.set_grouped(ModelRc::new(VecModel::from(Vec::<DiscoverSection>::new())));
    }
    state.set_shown(shown as i32);
    state.set_hires_count(hires_count as i32);
}
