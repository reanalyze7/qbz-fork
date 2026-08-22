//! Metadata fetching and FLAC tagging for cached tracks
//!
//! Handles complete metadata retrieval from Qobuz API and writing to FLAC tags.
//!
//! Split by responsibility: `model` (the DTO), `fetch` (Qobuz API calls),
//! `tags` (lofty tag writing), `artwork` (embed/save cover art), `filename`
//! (pure sanitizing), and `organize` (final path building + move).

mod artwork;
mod fetch;
mod filename;
mod model;
mod organize;
mod tags;

pub use artwork::{embed_artwork, save_album_artwork};
pub use fetch::fetch_complete_metadata;
pub use filename::sanitize_filename;
pub use model::CompleteTrackMetadata;
pub use organize::organize_cached_file;
pub use tags::write_flac_tags;
