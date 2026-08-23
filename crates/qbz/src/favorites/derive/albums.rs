use std::collections::HashMap;

use slint::{ComponentHandle, Model, ModelRc, VecModel};

use super::{album_alpha_key, album_genre_matches};
use crate::album_map;
use crate::favorites::albums_artwork::{
    dispatch_fav_albums_all_grouped, dispatch_fav_albums_all_visible, dispatch_fav_albums_window,
};
use crate::{AlbumCardItem, AlphaJump, AppWindow, DiscoverSection, FavoritesState};

/// Re-derive the rendered Albums list (`albums-visible`) from the full
/// `albums` set + the search query and sort key. Empty query + default
/// order shares the full model so artwork stays live; otherwise forks a
/// filtered + sorted clone (mirrors label.rs::derive_releases).
pub fn derive_albums(window: &AppWindow) {
    let state = window.global::<FavoritesState>();
    let query_owned = state.get_albums_search().to_lowercase();
    let query = query_owned.trim();
    let sort = state.get_albums_sort_by().to_string();
    let group = state.get_albums_group_mode().to_string();
    let genre_names = crate::genre_filter::selected_names("favorites");
    let all = state.get_albums();
    state.set_albums_alpha(ModelRc::new(VecModel::from(Vec::<AlphaJump>::new())));
    let empty_sections = || ModelRc::new(VecModel::from(Vec::<DiscoverSection>::new()));

    // Fast path: no filter, default order, no grouping, no genre -> share.
    if query.is_empty() && sort == "default" && group == "off" && genre_names.is_empty() {
        let n = all.row_count() as i32;
        state.set_albums_visible(all);
        state.set_albums_grouped(empty_sections());
        state.set_albums_shown(n);
        if state.get_albums_view_mode() == "list" {
            // The LIST view renders the same albums-visible model but is NOT
            // windowed (only AlbumGrid fires window-changed) — dispatch every
            // missing cover and do no eviction, like pre-windowing.
            dispatch_fav_albums_all_visible(window);
        } else {
            // The rows under the window band changed (fresh load / cleared
            // filter) but the band itself didn't — the grid won't re-fire,
            // so re-dispatch.
            dispatch_fav_albums_window(window);
        }
        return;
    }

    let mut filtered: Vec<AlbumCardItem> = (0..all.row_count())
        .filter_map(|i| all.row_data(i))
        .filter(|a| {
            (query.is_empty()
                || a.title.to_lowercase().contains(query)
                || a.artist.to_lowercase().contains(query))
                && album_genre_matches(a.genre.as_str(), &genre_names)
        })
        .collect();
    album_map::sort_album_items(&mut filtered, &sort);
    state.set_albums_shown(filtered.len() as i32);

    if group == "off" {
        state.set_albums_visible(ModelRc::new(VecModel::from(filtered)));
        state.set_albums_grouped(empty_sections());
        if state.get_albums_view_mode() == "list" {
            // Non-windowed list — full dispatch, no eviction (see above).
            dispatch_fav_albums_all_visible(window);
        } else {
            // The rows under the window band changed (search/sort/filter) but
            // the band itself didn't — the grid won't re-fire, so re-dispatch.
            dispatch_fav_albums_window(window);
        }
        return;
    }

    // Grouped: bucket by artist name, or by the title's first letter
    // (# bucket for non-alphabetic), sections ordered alphabetically.
    let mut map: Vec<(String, Vec<AlbumCardItem>)> = Vec::new();
    let mut index: HashMap<String, usize> = HashMap::new();
    for item in filtered {
        let key = if group == "artist" {
            let a = item.artist.to_string();
            if a.is_empty() {
                "Unknown".to_string()
            } else {
                a
            }
        } else {
            album_alpha_key(item.title.as_str())
        };
        let idx = *index.entry(key.clone()).or_insert_with(|| {
            map.push((key.clone(), Vec::new()));
            map.len() - 1
        });
        map[idx].1.push(item);
    }
    map.sort_by(|(a, _), (b, _)| match (a == "#", b == "#") {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.to_lowercase().cmp(&b.to_lowercase()),
    });
    // A-Z jump strip for alpha grouping: the section letters in order.
    if group == "alpha" {
        let alpha: Vec<AlphaJump> = map
            .iter()
            .enumerate()
            .map(|(i, (k, _))| AlphaJump {
                letter: k.clone().into(),
                index: i as i32,
            })
            .collect();
        state.set_albums_alpha(ModelRc::new(VecModel::from(alpha)));
    }
    let sections: Vec<DiscoverSection> = map
        .into_iter()
        .map(|(key, items)| DiscoverSection {
            title: key.into(),
            endpoint: "".into(),
            albums: ModelRc::new(VecModel::from(items)),
        })
        .collect();
    state.set_albums_grouped(ModelRc::new(VecModel::from(sections)));
    state.set_albums_visible(ModelRc::new(VecModel::from(Vec::<AlbumCardItem>::new())));
    dispatch_fav_albums_all_grouped(window);
}
