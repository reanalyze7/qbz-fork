//! Re-derive the flat-mode visible / grouped folder models.

use slint::{ComponentHandle, Model, ModelRc, VecModel};

use crate::{AlbumCardItem, AppWindow, DiscoverSection, LocalLibraryState};

use crate::local_library::shared::folder_alpha_key;

/// Re-derive the flat-mode visible / grouped folder models from the full
/// `folders` set, applying the toolbar's search query, sort key and group
/// mode. Mirrors `favorites::derive_albums` so behaviour is identical to the
/// metadata Albums tab; the only difference is the source model (directory
/// grouping instead of metadata grouping).
pub fn derive_folders(window: &AppWindow) {
    let s = window.global::<LocalLibraryState>();
    let query_owned = s.get_folders_search().to_lowercase();
    let query = query_owned.trim();
    let sort = s.get_folders_sort().to_string();
    let group = s.get_folders_group().to_string();
    let all = s.get_folders();
    let empty_sections = || ModelRc::new(VecModel::from(Vec::<DiscoverSection>::new()));

    let mut filtered: Vec<AlbumCardItem> = (0..all.row_count())
        .filter_map(|i| all.row_data(i))
        .filter(|a| {
            query.is_empty()
                || a.title.to_lowercase().contains(query)
                || a.artist.to_lowercase().contains(query)
        })
        .collect();
    crate::album_map::sort_album_items(&mut filtered, &sort);

    if group == "off" {
        s.set_folders_visible(ModelRc::new(VecModel::from(filtered)));
        s.set_folders_grouped(empty_sections());
        return;
    }

    // Grouped: bucket by artist name, or by the title's first letter
    // (`#` for non-alphabetic). Sections ordered alphabetically, `#` last.
    let mut map: Vec<(String, Vec<AlbumCardItem>)> = Vec::new();
    let mut index: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for item in filtered {
        let key = if group == "artist" {
            let a = item.artist.to_string();
            if a.is_empty() {
                qbz_i18n::t("Unknown")
            } else {
                a
            }
        } else {
            folder_alpha_key(item.title.as_str())
        };
        let idx = *index.entry(key.clone()).or_insert_with(|| {
            map.push((key.clone(), Vec::new()));
            map.len() - 1
        });
        map[idx].1.push(item);
    }
    map.sort_by(|(a, _), (b, _)| match (a == "#", b == "#") {
        (true, false) => std::cmp::Ordering::Greater,
        (false, true) => std::cmp::Ordering::Less,
        _ => a.to_lowercase().cmp(&b.to_lowercase()),
    });
    let sections: Vec<DiscoverSection> = map
        .into_iter()
        .map(|(key, items)| DiscoverSection {
            title: key.into(),
            endpoint: "".into(),
            albums: ModelRc::new(VecModel::from(items)),
        })
        .collect();
    s.set_folders_grouped(ModelRc::new(VecModel::from(sections)));
    s.set_folders_visible(ModelRc::new(VecModel::from(Vec::<AlbumCardItem>::new())));
}
