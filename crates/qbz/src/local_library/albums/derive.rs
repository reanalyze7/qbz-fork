//! Re-derive the rendered Albums sets (search + filter + sort + group + A-Z)
//! from the full loaded set.

use slint::{ComponentHandle, Model, ModelRc, VecModel};

use crate::{AlbumCardItem, AlphaJump, AppWindow, DiscoverSection, LocalLibraryState};

use super::artwork::{dispatch_albums_all_grouped, dispatch_albums_all_visible, dispatch_albums_window};
use super::filter::{album_filter_count, album_matches_filters, read_album_filter};
use super::load::local_albums;
use crate::local_library::shared::folder_alpha_key;

/// Re-derive the rendered Albums sets (search + quality/format/source filter +
/// sort + group + A-Z) from the full `albums` card set, filtered by id against
/// the raw LocalAlbum cache. Mirrors `derive_folders` plus the filter.
pub fn derive_albums(window: &AppWindow) {
    let s = window.global::<LocalLibraryState>();
    let query_owned = s.get_albums_search().to_lowercase();
    let query = query_owned.trim();
    let sort = s.get_albums_sort().to_string();
    let group = s.get_albums_group().to_string();
    let filter = read_album_filter(window);
    window
        .global::<crate::LibAlbumFilterState>()
        .set_count(album_filter_count(&filter));

    let matching: std::collections::HashSet<String> = {
        let cache = local_albums();
        cache
            .iter()
            .filter(|a| {
                (query.is_empty()
                    || a.title.to_lowercase().contains(query)
                    || a.artist.to_lowercase().contains(query)
                    || a.all_artists.to_lowercase().contains(query))
                    && album_matches_filters(a, &filter)
            })
            .map(|a| a.id.clone())
            .collect()
    };

    let all = s.get_albums();
    let mut filtered: Vec<AlbumCardItem> = (0..all.row_count())
        .filter_map(|i| all.row_data(i))
        .filter(|c| matching.contains(&c.id.to_string()))
        .collect();
    crate::album_map::sort_album_items(&mut filtered, &sort);
    s.set_albums_shown(filtered.len() as i32);

    let empty_sections = || ModelRc::new(VecModel::from(Vec::<DiscoverSection>::new()));
    if group == "off" {
        s.set_albums_visible(ModelRc::new(VecModel::from(filtered)));
        s.set_albums_grouped(empty_sections());
        s.set_albums_alpha(ModelRc::new(VecModel::from(Vec::<AlphaJump>::new())));
        if s.get_albums_view_mode() == "list" {
            // The LIST view renders the same albums-visible model but is NOT
            // windowed (only AlbumGrid fires window-changed) — dispatch every
            // missing cover and do no eviction, like pre-windowing.
            dispatch_albums_all_visible(window);
        } else {
            // The rows under the window band changed (search/sort/filter) but
            // the band itself didn't — the grid won't re-fire, so re-dispatch.
            dispatch_albums_window(window);
        }
        return;
    }

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
    let jumps: Vec<AlphaJump> = map
        .iter()
        .enumerate()
        .map(|(i, (k, _))| AlphaJump {
            letter: k.clone().into(),
            index: i as i32,
        })
        .collect();
    let sections: Vec<DiscoverSection> = map
        .into_iter()
        .map(|(key, items)| DiscoverSection {
            title: key.into(),
            endpoint: "".into(),
            albums: ModelRc::new(VecModel::from(items)),
        })
        .collect();
    s.set_albums_grouped(ModelRc::new(VecModel::from(sections)));
    s.set_albums_alpha(ModelRc::new(VecModel::from(jumps)));
    s.set_albums_visible(ModelRc::new(VecModel::from(Vec::<AlbumCardItem>::new())));
    dispatch_albums_all_grouped(window);
}
