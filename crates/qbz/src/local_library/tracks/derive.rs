//! Group-by + search derive for the Tracks tab, plus the group/sort setters.

use slint::{ComponentHandle, Model, ModelRc, VecModel};

use crate::{AlphaJump, AppWindow, LocalLibraryState, TrackItem};

use crate::local_library::shared::folder_alpha_key;

/// Re-derive the group-ordered + search-filtered `tracks-visible` render model
/// from the loaded `tracks`, plus the A-Z jump strip for name grouping. Uses
/// the local `folder_alpha_key`; no genre filter (no local genre surface).
pub fn derive_tracks(window: &AppWindow) {
    let s = window.global::<LocalLibraryState>();
    let query_owned = s.get_tracks_search().to_lowercase();
    let query = query_owned.trim();
    let group = s.get_tracks_group_mode().to_string();
    let all = s.get_tracks();
    s.set_tracks_alpha(ModelRc::new(VecModel::from(Vec::<AlphaJump>::new())));

    // Fast path: no search + no grouping -> share the loaded model.
    if query.is_empty() && group == "off" {
        s.set_tracks_visible(all);
        return;
    }
    let mut filtered: Vec<TrackItem> = (0..all.row_count())
        .filter_map(|i| all.row_data(i))
        .filter(|t| {
            query.is_empty()
                || t.title.to_lowercase().contains(query)
                || t.artist.to_lowercase().contains(query)
                || t.album.to_lowercase().contains(query)
        })
        .collect();
    let lc = |s: &slint::SharedString| s.to_lowercase();
    match group.as_str() {
        "album" => filtered
            .sort_by(|a, b| lc(&a.album).cmp(&lc(&b.album)).then(lc(&a.title).cmp(&lc(&b.title)))),
        "artist" => filtered.sort_by(|a, b| {
            lc(&a.artist)
                .cmp(&lc(&b.artist))
                .then(lc(&a.album).cmp(&lc(&b.album)))
                .then(lc(&a.title).cmp(&lc(&b.title)))
        }),
        "name" => filtered.sort_by(|a, b| lc(&a.title).cmp(&lc(&b.title))),
        _ => {}
    }
    if group == "name" {
        let mut jumps: Vec<AlphaJump> = Vec::new();
        let mut last = String::new();
        for (i, t) in filtered.iter().enumerate() {
            let key = folder_alpha_key(t.title.as_str());
            if key != last {
                jumps.push(AlphaJump {
                    letter: key.clone().into(),
                    index: i as i32,
                });
                last = key;
            }
        }
        s.set_tracks_alpha(ModelRc::new(VecModel::from(jumps)));
    }
    s.set_tracks_visible(ModelRc::new(VecModel::from(filtered)));
}

/// Set the group mode, persist it, re-derive.
pub fn set_tracks_group(window: &AppWindow, mode: &str) {
    window.global::<LocalLibraryState>().set_tracks_group_mode(mode.into());
    crate::locallibrary_prefs::save(window);
    derive_tracks(window);
}

/// Set the SQL sort key, persist it, re-query page 1. Sort is server-side
/// (it defines the pagination order), so unlike group-by this is a
/// gen-bumping page-1 reload, not a client-side re-derive.
pub fn set_tracks_sort(window: &AppWindow, key: &str, handle: tokio::runtime::Handle) {
    window.global::<LocalLibraryState>().set_tracks_sort(key.into());
    crate::locallibrary_prefs::save(window);
    super::load::reload_tracks(window.as_weak(), handle);
}
