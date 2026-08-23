use slint::{ComponentHandle, Model, ModelRc, VecModel};

use super::{album_alpha_key, track_genre_matches};
use crate::{AlphaJump, AppWindow, FavoritesState, TrackItem};

/// Re-derive the rendered Tracks list (`tracks-visible`) from the full
/// `tracks` set and the search query. An empty query shares the full
/// model so artwork keeps updating in place (the LabelState albums/visible
/// pattern); a query forks a filtered clone (each row carries its already
/// decoded artwork, so no re-fetch).
pub fn derive_tracks(window: &AppWindow) {
    let state = window.global::<FavoritesState>();
    let query_owned = state.get_tracks_search().to_lowercase();
    let query = query_owned.trim();
    let group = state.get_tracks_group_mode().to_string();
    let genre_names = crate::genre_filter::selected_names("favorites");
    let all = state.get_tracks();
    state.set_tracks_alpha(ModelRc::new(VecModel::from(Vec::<AlphaJump>::new())));
    // Fast path: no search + no grouping + no genre filter -> share model.
    if query.is_empty() && group == "off" && genre_names.is_empty() {
        state.set_tracks_visible(all);
        return;
    }
    let mut filtered: Vec<TrackItem> = (0..all.row_count())
        .filter_map(|i| all.row_data(i))
        .filter(|t| {
            (query.is_empty()
                || t.title.to_lowercase().contains(query)
                || t.artist.to_lowercase().contains(query)
                || t.album.to_lowercase().contains(query))
                && track_genre_matches(t.id.as_str(), &genre_names)
        })
        .collect();
    // Group-by reorders the rows so a group's tracks sit together (Tauri
    // adds visible headers; v1 here is group-ordering without header rows
    // until the list is virtualized).
    let lc = |s: &slint::SharedString| s.to_lowercase();
    match group.as_str() {
        "album" => {
            filtered.sort_by(|a, b| lc(&a.album).cmp(&lc(&b.album)).then(lc(&a.title).cmp(&lc(&b.title))))
        }
        "artist" => filtered.sort_by(|a, b| {
            lc(&a.artist)
                .cmp(&lc(&b.artist))
                .then(lc(&a.album).cmp(&lc(&b.album)))
                .then(lc(&a.title).cmp(&lc(&b.title)))
        }),
        "name" => filtered.sort_by(|a, b| lc(&a.title).cmp(&lc(&b.title))),
        _ => {}
    }
    // A-Z jump strip for name grouping: first row index per distinct initial.
    if group == "name" {
        let mut jumps: Vec<AlphaJump> = Vec::new();
        let mut last = String::new();
        for (i, t) in filtered.iter().enumerate() {
            let key = album_alpha_key(t.title.as_str());
            if key != last {
                jumps.push(AlphaJump {
                    letter: key.clone().into(),
                    index: i as i32,
                });
                last = key;
            }
        }
        state.set_tracks_alpha(ModelRc::new(VecModel::from(jumps)));
    }
    state.set_tracks_visible(ModelRc::new(VecModel::from(filtered)));
}
