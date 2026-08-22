//! Resolve a recommendation candidate to a real Qobuz entity (track/artist/album).
//!
//! Tracks: ISRC -> MusicBrainz `inc=isrcs` -> Qobuz, else fuzzy text.
//! Artists: Qobuz artist-search + normalized-name match.
//! Albums: UPC match if known, else fuzzy text (title*0.6 + artist*0.4).
//! Every outcome (positive AND negative) is cached.

use std::sync::Mutex;

use crate::cache::RecoCache;

mod album;
mod album_filters;
mod artist;
mod track;
mod track_resolve;

pub use album::{build_album_reco, is_full_album, is_slop, validate_album};
pub use artist::validate_artist;
pub use track::validate_track;

type Cache<'a> = Option<&'a Mutex<RecoCache>>;
