//! Build the artist→album→track rollup + rows from a flat track list,
//! applying the current toolbar filters.

use qbz_offline_cache::CachedTrackInfo;

use crate::OfflineArtist;

use super::filters::Filters;
use super::row::RowData;

mod albums;
mod rows;

/// The rollup result: the artist rail + the flat interleaved row list.
pub(super) struct Rollup {
    pub artists: Vec<OfflineArtist>,
    pub rows: Vec<RowData>,
}

pub(super) fn build(tracks: Vec<CachedTrackInfo>, cache_path: &str, f: &Filters) -> Rollup {
    let grouped = albums::group(tracks);
    let artists = albums::artist_rail(&grouped.order, &grouped.albums, f);
    let rows = rows::build(&grouped.order, &grouped.albums, cache_path, f);
    Rollup { artists, rows }
}
