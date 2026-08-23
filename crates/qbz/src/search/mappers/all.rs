use std::collections::HashSet;

use qbz_models::{MostPopularItem, SearchAllResults};

use super::album_track::{map_album, map_track};
use super::artist_playlist::map_artist;
use crate::search::rows::{ArtistRow, MostPopularRow, SearchData};

pub(crate) fn map_most_popular(
    item: Option<MostPopularItem>,
    favorite_artists: &HashSet<u64>,
) -> MostPopularRow {
    match item {
        Some(MostPopularItem::Albums(a)) => MostPopularRow::Album(map_album(a)),
        Some(MostPopularItem::Artists(a)) => {
            let following = favorite_artists.contains(&a.id);
            MostPopularRow::Artist(map_artist(&a, following))
        }
        Some(MostPopularItem::Tracks(t)) => MostPopularRow::Track(map_track(t)),
        None => MostPopularRow::None,
    }
}

/// Map a combined-search result into plain `Send` data. `favorite_artists`
/// is the set of artist ids the user already follows.
pub fn map_search_all(
    query: &str,
    results: SearchAllResults,
    favorite_artists: &HashSet<u64>,
) -> SearchData {
    let artists: Vec<ArtistRow> = results
        .artists
        .items
        .iter()
        .map(|a| map_artist(a, favorite_artists.contains(&a.id)))
        .collect();
    let most_popular = map_most_popular(results.most_popular, favorite_artists);
    // Dedupe used to drop the top-result artist from the artists list
    // here, but the Artists tab does not show the Most-popular hero —
    // it should keep the artist. The dedupe now lives at `apply_search`
    // where the carousel-only `artists_carousel` is built.
    SearchData {
        query: query.to_string(),
        albums_total: results.albums.total,
        tracks_total: results.tracks.total,
        artists_total: results.artists.total,
        playlists_total: results.playlists.total,
        albums: results.albums.items.into_iter().map(map_album).collect(),
        tracks: results.tracks.items.into_iter().map(map_track).collect(),
        artists,
        playlists: results.playlists.items.into_iter().map(super::artist_playlist::map_playlist).collect(),
        most_popular,
    }
}
