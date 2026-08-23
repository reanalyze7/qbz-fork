//! Re-derive the Artists left-rail render sets, and apply a freshly-merged
//! row set to the model.

use slint::{ComponentHandle, Model, ModelRc, VecModel};

use crate::{AlphaJump, AppWindow, LocalArtistItem, LocalArtistSection, LocalLibraryState};

use crate::local_library::shared::folder_alpha_key;

use super::merge::ArtistRow;

/// Re-derive the left-rail render sets (search filter + A-Z grouping + jump
/// strip) from the merged `artists` master list.
pub fn derive_artists(window: &AppWindow) {
    let s = window.global::<LocalLibraryState>();
    let query_owned = s.get_artists_search().to_lowercase();
    let query = query_owned.trim();
    let all = s.get_artists();
    let filtered: Vec<LocalArtistItem> = (0..all.row_count())
        .filter_map(|i| all.row_data(i))
        .filter(|a| query.is_empty() || a.display_name.to_lowercase().contains(query))
        .collect();
    s.set_artists_shown(filtered.len() as i32);

    let mut map: Vec<(String, Vec<LocalArtistItem>)> = Vec::new();
    let mut index: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for item in filtered {
        let key = folder_alpha_key(item.display_name.as_str());
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
    let sections: Vec<LocalArtistSection> = map
        .into_iter()
        .map(|(letter, artists)| LocalArtistSection {
            letter: letter.into(),
            artists: ModelRc::new(VecModel::from(artists)),
        })
        .collect();
    s.set_artists_grouped(ModelRc::new(VecModel::from(sections)));
    s.set_artists_alpha(ModelRc::new(VecModel::from(jumps)));
}

pub(crate) fn apply_artists(window: &AppWindow, rows: Vec<ArtistRow>) {
    // Build the Slint items here (UI thread) — `LocalArtistItem.image` holds a
    // non-Send `slint::Image`, so the rows crossed `spawn_blocking` as the
    // Send-safe `ArtistRow` and gain the (default-empty) decoded image now.
    let items: Vec<LocalArtistItem> = rows
        .into_iter()
        .map(|r| LocalArtistItem {
            name: r.name.into(),
            display_name: r.display_name.into(),
            album_count: r.album_count,
            track_count: r.track_count,
            image_path: r.image_path.into(),
            image: slint::Image::default(),
        })
        .collect();
    let s = window.global::<LocalLibraryState>();
    s.set_artists(ModelRc::new(VecModel::from(items)));
    s.set_artists_loading(false);
    s.set_artists_load_failed(false);
    derive_artists(window);
}
