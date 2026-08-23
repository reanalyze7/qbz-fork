//! Recently-played store.
//!
//! A small JSON file at the shared QBZ data path holding the last few
//! played tracks AND the last few played albums, newest first. Discover
//! Home renders two sections from it — recently-played tracks (slim
//! cards) and recently-played albums. The playback session calls
//! [`record`] when a track starts.
//!
//! The album history is a SEPARATE list with its own cap (#567): deriving
//! albums from the 24-track window collapsed long albums into ~4 distinct
//! album cards, starving the "Recently Played Albums" rail. Both lists are
//! deduplicated by id at record time. Persisted format is an object
//! `{ "tracks": [...], "albums": [...] }`; the legacy format (a bare track
//! array) is migrated on read by deriving the album list from the track
//! window exactly as before, so old stores lose nothing.
//!
//! Until playback is wired the store is simply empty and the Home
//! sections that read it hide themselves — the data path exists end to
//! end so playback only has to call `record`.

mod album_meta;
mod api;
mod model;
mod store_io;

pub use album_meta::{album_meta, remember_album_meta, AlbumMeta};
pub use api::{load, load_albums, prune_albums, record};
pub use model::{RecentAlbum, RecentTrack};
