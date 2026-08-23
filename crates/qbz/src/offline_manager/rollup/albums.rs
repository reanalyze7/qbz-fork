//! Group the flat track list into albums, and build the artist rail +
//! the sorted album display order.

use std::collections::BTreeMap;

use qbz_offline_cache::CachedTrackInfo;

use crate::OfflineArtist;

use super::super::filters::Filters;
use super::super::format::album_size;

pub(super) type AlbumsMap = BTreeMap<String, (String, String, Vec<CachedTrackInfo>)>;

pub(super) struct Grouped {
    pub order: Vec<String>,
    pub albums: AlbumsMap,
}

/// Group tracks by album_id ("__singles__" when absent), first-seen order
/// (the DB already returns rows most-recently-accessed first, so this order
/// is the "recent" sort).
pub(super) fn group(tracks: Vec<CachedTrackInfo>) -> Grouped {
    let mut order: Vec<String> = Vec::new();
    let mut albums: AlbumsMap = BTreeMap::new();
    for t in tracks {
        let aid = t.album_id.clone().unwrap_or_else(|| "__singles__".to_string());
        if !albums.contains_key(&aid) {
            order.push(aid.clone());
        }
        let title = t.album.clone().unwrap_or_else(|| "Singles".to_string());
        albums
            .entry(aid)
            .or_insert_with(|| (t.artist.clone(), title, Vec::new()))
            .2
            .push(t);
    }
    Grouped { order, albums }
}

/// Artist rail: name -> (album_count, track_count), A-Z (BTreeMap order).
pub(super) fn artist_rail(order: &[String], albums: &AlbumsMap, f: &Filters) -> Vec<OfflineArtist> {
    let mut artist_stats: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    for aid in order {
        let (artist, _title, group) = &albums[aid];
        let e = artist_stats.entry(artist.clone()).or_insert((0, 0));
        e.0 += 1;
        e.1 += group.len();
    }
    artist_stats
        .iter()
        .map(|(name, (albums_n, tracks_n))| OfflineArtist {
            name: name.clone().into(),
            meta: qbz_i18n::t_args(
                "{} albums · {} tracks",
                &[&albums_n.to_string(), &tracks_n.to_string()],
            )
            .into(),
            selected: *name == f.selected_artist,
        })
        .collect()
}

/// Album display order per the sort.
pub(super) fn sorted_order(order: &[String], albums: &AlbumsMap, sort: i32) -> Vec<String> {
    let mut order = order.to_vec();
    match sort {
        0 => order.sort_by(|a, b| albums[a].1.to_lowercase().cmp(&albums[b].1.to_lowercase())),
        2 => order.sort_by(|a, b| album_size(&albums[b].2).cmp(&album_size(&albums[a].2))),
        3 => order.sort_by(|a, b| album_size(&albums[a].2).cmp(&album_size(&albums[b].2))),
        _ => {} // 1 recent — keep the DB's last_accessed_at DESC order
    }
    order
}
