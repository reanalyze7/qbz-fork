//! B9 — offline Favorites "playable favorites" rail.
//!
//! While OFFLINE the Favorites view mounts the shared OfflinePlaceholder;
//! this module fills the rail under the placeholder copy with the favorite
//! tracks that are still playable. Three local id/metadata sources:
//!
//!   favorites      — `fav_cache` (disk-first seeded favorites_cache.db,
//!                    so the set is correct with zero network)
//!   offline cache  — qbz-offline-cache index rows with status READY
//!   library copies — library.db `local_tracks` rows with
//!                    `source = 'qobuz_download'` (the Local Library
//!                    "Offline" source-filter set)
//!
//! rail = favorites ∩ (ready ∪ qobuz_download). Metadata comes from the
//! index row when present (title/artist columns + the offline cover chain
//! via [`CachedTrackInfo::resolve_cover_path`]), else from the library row;
//! ids with no local metadata are skipped (count logged). Zero schema
//! changes — both stores are read as-is.
//!
//! A row click replaces the player queue with the WHOLE rail starting at
//! the clicked row; the tracks carry the real Qobuz id +
//! `source = "qobuz_download"` (the `local_queue_track` offline-copy
//! shape), so playback runs through the existing offline cache tier.

mod gather;
mod load;
mod play;
mod state;

pub use load::load;
pub use play::play;
