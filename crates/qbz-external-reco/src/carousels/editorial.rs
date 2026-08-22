//! Cold-start editorial (top albums + artists).

use std::collections::HashSet;

use futures_util::stream::{self, StreamExt};

use crate::types::{AlbumReco, ArtistReco, RecoSource};
use crate::validate::build_album_reco;
use crate::RecoInputs;

pub async fn build_editorial(inputs: &RecoInputs<'_>) -> (Vec<AlbumReco>, Vec<ArtistReco>) {
    let (most_streamed, new_releases) = tokio::join!(
        inputs.catalog.featured_albums("most-streamed", 20),
        inputs.catalog.featured_albums("new-releases", 20),
    );

    let mut seen_albums: HashSet<String> = HashSet::new();
    let mut top_albums: Vec<AlbumReco> = Vec::new();
    for album in most_streamed.iter().chain(new_releases.iter()) {
        if !album.id.is_empty() && seen_albums.insert(album.id.clone()) {
            top_albums.push(build_album_reco(album, String::new(), RecoSource::Editorial));
        }
    }
    top_albums.truncate(20);

    let mut seen_artists: HashSet<u64> = HashSet::new();
    let mut artist_ids: Vec<(u64, String)> = Vec::new();
    for album in most_streamed.iter().chain(new_releases.iter()) {
        let id = album.artist.id;
        if id != 0 && seen_artists.insert(id) {
            artist_ids.push((id, album.artist.name.clone()));
        }
        if artist_ids.len() >= 12 {
            break;
        }
    }
    let top_artists: Vec<ArtistReco> = stream::iter(artist_ids.into_iter().map(|(id, name)| {
        let catalog = inputs.catalog;
        async move {
            let image_url = catalog
                .get_artist(id)
                .await
                .and_then(|a| a.image.and_then(|i| i.best().cloned()))
                .unwrap_or_default();
            ArtistReco {
                qobuz_artist_id: id,
                name,
                image_url,
                subtitle: String::new(),
                source: RecoSource::Editorial,
            }
        }
    }))
    .buffered(4)
    .collect()
    .await;

    (top_albums, top_artists)
}
