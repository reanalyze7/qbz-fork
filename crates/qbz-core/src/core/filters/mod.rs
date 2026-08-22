//! Pure blacklist-filtering and search-page-parsing helpers. No
//! `QbzCore` dependency — free functions only, easily unit tested.

mod paging;
mod predicates;

#[cfg(test)]
mod tests;

pub use predicates::{album_blacklisted, discover_album_blacklisted, track_blacklisted};
use paging::{parse_page, pick_most_popular};

use qbz_models::{Album, Artist, Playlist, SearchAllResults, Track};

use super::{AlbumBlacklistFilter, BlacklistFilter};

/// Parse a `catalog_search` JSON payload into typed category pages,
/// dropping any item whose artist id is blacklisted and adjusting totals.
pub(crate) fn parse_search_all(
    value: &serde_json::Value,
    blacklist: &BlacklistFilter,
    album_bl: &AlbumBlacklistFilter,
) -> SearchAllResults {
    let mut albums = parse_page::<Album>(value, "albums");
    let mut tracks = parse_page::<Track>(value, "tracks");
    let mut artists = parse_page::<Artist>(value, "artists");
    let playlists = parse_page::<Playlist>(value, "playlists");

    // Artists have no album id — artist axis only.
    let before = artists.items.len();
    artists.items.retain(|a| !blacklist.contains(&a.id));
    artists.total = artists
        .total
        .saturating_sub((before - artists.items.len()) as u32);

    let before = albums.items.len();
    albums.items.retain(|al| !album_blacklisted(al, blacklist, album_bl));
    albums.total = albums
        .total
        .saturating_sub((before - albums.items.len()) as u32);

    let before = tracks.items.len();
    tracks
        .items
        .retain(|track| !track_blacklisted(track, blacklist, album_bl));
    tracks.total = tracks
        .total
        .saturating_sub((before - tracks.items.len()) as u32);

    SearchAllResults {
        albums,
        tracks,
        artists,
        playlists,
        most_popular: pick_most_popular(value, blacklist, album_bl),
    }
}
