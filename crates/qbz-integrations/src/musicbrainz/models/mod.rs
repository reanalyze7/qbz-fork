//! MusicBrainz API response models
//!
//! Types for deserializing MusicBrainz JSON responses, split by concern
//! (confidence/classification, recording, artist, release, area,
//! relationships, resolved output types, discovery, musician).

mod area;
mod artist;
mod confidence;
mod discovery;
mod musician;
mod recording;
mod relationships;
mod release;
mod resolved;

pub use area::*;
pub use artist::*;
pub use confidence::*;
pub use discovery::*;
pub use musician::*;
pub use recording::*;
pub use relationships::*;
pub use release::*;
pub use resolved::*;
