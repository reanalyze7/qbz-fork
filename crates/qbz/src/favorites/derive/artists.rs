use std::collections::HashMap;

use slint::{ComponentHandle, Model, ModelRc, VecModel};

use super::album_alpha_key;
use crate::{AlphaJump, AppWindow, FavArtistSection, FavoriteArtistItem, FavoritesState};

/// Re-derive the rendered Artists grid (`artists-visible`) from the full
/// `artists` set + the search query (name substring; mirrors Tauri's
/// filteredArtists). A-Z grouping + the alpha strip are layered on later.
pub fn derive_artists(window: &AppWindow) {
    let state = window.global::<FavoritesState>();
    let query_owned = state.get_artists_search().to_lowercase();
    let query = query_owned.trim();
    // The sidepanel left list is ALWAYS A-Z grouped (independent of the grid
    // group toggle), so it shows letter headers + the alpha jump strip.
    let group = state.get_artists_group_enabled()
        || state.get_artists_view_mode().as_str() == "sidepanel";
    let all = state.get_artists();

    // Flat (search-filtered) model. Share `all` when no query so artwork
    // keeps updating in place; a query forks a filtered clone.
    let filtered: Vec<FavoriteArtistItem> = if query.is_empty() {
        (0..all.row_count()).filter_map(|i| all.row_data(i)).collect()
    } else {
        (0..all.row_count())
            .filter_map(|i| all.row_data(i))
            .filter(|a| a.name.to_lowercase().contains(query))
            .collect()
    };
    state.set_artists_shown(filtered.len() as i32);
    if query.is_empty() {
        state.set_artists_visible(all);
    } else {
        state.set_artists_visible(ModelRc::new(VecModel::from(filtered.clone())));
    }

    // A-Z grouping (grid grouped mode): bucket by first letter, sections
    // ordered (# first then A-Z), with an alpha jump per section.
    if !group {
        state.set_artists_grouped(ModelRc::new(VecModel::from(Vec::<FavArtistSection>::new())));
        state.set_artists_alpha(ModelRc::new(VecModel::from(Vec::<AlphaJump>::new())));
        return;
    }
    let mut sorted = filtered;
    sorted.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    let mut map: Vec<(String, Vec<FavoriteArtistItem>)> = Vec::new();
    let mut index: HashMap<String, usize> = HashMap::new();
    for item in sorted {
        let key = album_alpha_key(item.name.as_str());
        let idx = *index.entry(key.clone()).or_insert_with(|| {
            map.push((key.clone(), Vec::new()));
            map.len() - 1
        });
        map[idx].1.push(item);
    }
    map.sort_by(|(a, _), (b, _)| match (a == "#", b == "#") {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.cmp(b),
    });
    let alpha: Vec<AlphaJump> = map
        .iter()
        .enumerate()
        .map(|(i, (k, _))| AlphaJump {
            letter: k.clone().into(),
            index: i as i32,
        })
        .collect();
    let sections: Vec<FavArtistSection> = map
        .into_iter()
        .map(|(key, artists)| FavArtistSection {
            key: key.clone().into(),
            title: key.into(),
            artists: ModelRc::new(VecModel::from(artists)),
        })
        .collect();
    state.set_artists_alpha(ModelRc::new(VecModel::from(alpha)));
    state.set_artists_grouped(ModelRc::new(VecModel::from(sections)));
}
