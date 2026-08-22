//! Configurable weights for different relationship types
//!
//! These weights determine how strongly different types of relationships
//! contribute to artist similarity vectors.

mod lookup;
mod presets;
#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

/// Weights for different relationship types when building artist vectors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipWeights {
    // === MusicBrainz artist-to-artist relationships ===
    /// Weight for band membership (artist was/is member of group)
    /// Strongest connection - same creative unit
    pub member_of_band: f32,

    /// Weight for collaboration (artists worked together)
    pub collaboration: f32,

    /// Weight for "is person" in a group (reverse of member_of_band)
    pub has_member: f32,

    /// Weight for founder relationship
    pub founder: f32,

    // === MusicBrainz recording/release relationships ===
    /// Weight for performer credit on a recording
    pub performer: f32,

    /// Weight for composer credit
    pub composer: f32,

    /// Weight for producer credit
    pub producer: f32,

    /// Weight for conductor credit
    pub conductor: f32,

    /// Weight for engineer/mixer credit
    pub engineer: f32,

    // === Qobuz relationships ===
    /// Weight for Qobuz similar artists
    pub qobuz_similar: f32,

    // === Tag-based relationships ===
    /// Weight for shared MusicBrainz tags (genres)
    pub shared_tag: f32,

    // === Behavioral relationships ===
    /// Weight for user listening affinity (artists played together)
    pub user_affinity: f32,
}
