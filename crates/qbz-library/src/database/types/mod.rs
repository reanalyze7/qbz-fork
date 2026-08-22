//! Shared data types for the `database` module, split out of the original
//! monolithic `database.rs` so each domain submodule can depend on them
//! without cross-file duplication. Re-exported at `crate::database::*` from
//! `database/mod.rs` so the public API path is unchanged.

mod misc;
mod playlist_meta;
mod playlist_settings;

pub use misc::{AlbumTrackUpdate, LibraryFolder, LibraryStats, TrackMetadataUpdateFull};
pub use playlist_meta::{PlaylistFolder, PlaylistStats};
pub use playlist_settings::{LocalContentStatus, PlaylistSettings};
