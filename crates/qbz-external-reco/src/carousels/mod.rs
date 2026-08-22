//! Per-row candidate generation, blending, filtering, Qobuz validation, rotation.
//!
//! The documented Last.fm "artist discovery" recipe (api-evangelist/lastfm
//! arazzo workflow): top artists -> artist.getSimilar -> top albums. There is no
//! recommendation endpoint, so recommendations are replicated from similarity.

use std::sync::Mutex;

use crate::cache::RecoCache;
use crate::matching::normalize;

mod artist_rails;
mod artist_rows;
mod albums_rec;
mod albums_seeded;
mod deep_cuts;
mod editorial;
mod fresh_releases;
mod history;
#[cfg(test)]
mod tests;
mod validate_pools;
mod weekly;
mod weekly_discover;

pub use artist_rails::{compose_artist_rails, ArtistRailComposition};
pub use artist_rows::{build_rec_artists_common, build_rec_artists_recent};
pub use albums_rec::build_rec_albums;
pub use albums_seeded::build_similar_albums_seeded;
pub use deep_cuts::build_deep_cut_albums;
pub use editorial::build_editorial;
pub use fresh_releases::build_fresh_releases;
pub use history::gather_history;
pub use weekly::build_weekly;

const DISPLAY_CAP: usize = 20;
/// Public alias for the paint layer (it composes the visible rows after
/// filtering/dedup, so it needs the canonical per-rail visible cap).
pub const ARTIST_DISPLAY_CAP: usize = DISPLAY_CAP;
const PLAYLIST_CAP: usize = 30;
const VALIDATE_CONCURRENCY: usize = 6;
const ARTIST_SEEDS: usize = 6;
const SIMILAR_PER_SEED: u32 = 12;
const KNOWN_ARTISTS_PER_BUILD: usize = 50;

type Cache<'a> = Option<&'a Mutex<RecoCache>>;

fn track_key(artist: &str, title: &str) -> String {
    format!("{}|{}", normalize(artist), normalize(title))
}
fn album_key(artist: &str, album: &str) -> String {
    format!("{}|{}", normalize(artist), normalize(album))
}

fn rotate_take<T>(mut pool: Vec<T>, seed: u64, take: usize) -> Vec<T> {
    if pool.is_empty() {
        return pool;
    }
    let off = (seed as usize) % pool.len();
    pool.rotate_left(off);
    pool.truncate(take);
    pool
}
