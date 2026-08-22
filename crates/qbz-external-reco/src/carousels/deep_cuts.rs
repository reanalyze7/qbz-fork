//! Deep-cut albums from artists you know (Qobuz catalog, not heard).

use std::collections::HashSet;

use futures_util::stream::{self, StreamExt};
use qbz_models::Album;

use crate::types::{AlbumReco, RecoSource};
use crate::validate::{build_album_reco, is_full_album, is_slop};
use crate::RecoInputs;

use super::{rotate_take, DISPLAY_CAP, KNOWN_ARTISTS_PER_BUILD};

pub async fn build_deep_cut_albums(inputs: &RecoInputs<'_>) -> Vec<AlbumReco> {
    if inputs.local.known_artist_ids.is_empty() {
        return Vec::new();
    }
    let mut ids: Vec<u64> = inputs.local.known_artist_ids.iter().copied().collect();
    ids.sort_unstable();
    let ids = rotate_take(ids, inputs.rotation_seed, KNOWN_ARTISTS_PER_BUILD);

    let per_artist: Vec<Vec<Album>> = stream::iter(ids.into_iter().map(|id| {
        let catalog = inputs.catalog;
        async move { catalog.artist_albums(id, 12).await }
    }))
    .buffered(4)
    .collect()
    .await;

    let mut seen: HashSet<String> = HashSet::new();
    let mut pool: Vec<AlbumReco> = Vec::new();
    for albums in per_artist {
        for album in albums.into_iter().skip(2) {
            if album.id.is_empty()
                || !is_full_album(&album)
                || is_slop(&album.artist.name, &album.title)
                || inputs.local.played_album_ids.contains(&album.id)
                || !seen.insert(album.id.clone())
            {
                continue;
            }
            let subtitle = format!("Deep cut · {}", album.artist.name);
            pool.push(build_album_reco(&album, subtitle, RecoSource::Internal));
        }
    }
    rotate_take(pool, inputs.rotation_seed, DISPLAY_CAP)
}
