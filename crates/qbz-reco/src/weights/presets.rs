//! Preset weight configurations for `RelationshipWeights`.

use super::RelationshipWeights;

impl Default for RelationshipWeights {
    fn default() -> Self {
        Self {
            // Band relationships - strongest
            member_of_band: 1.0,
            has_member: 0.9,
            founder: 0.85,
            collaboration: 0.8,

            // Credit relationships - medium
            performer: 0.6,
            composer: 0.55,
            producer: 0.5,
            conductor: 0.5,
            engineer: 0.3,

            // Qobuz similarity - good signal
            qobuz_similar: 0.7,

            // Tags - weak but useful
            shared_tag: 0.3,

            // User behavior - medium
            user_affinity: 0.5,
        }
    }
}

impl RelationshipWeights {
    /// Create weights optimized for discovering band-related artists
    pub fn band_focused() -> Self {
        Self {
            member_of_band: 1.0,
            has_member: 1.0,
            founder: 0.9,
            collaboration: 0.7,
            performer: 0.4,
            composer: 0.3,
            producer: 0.2,
            conductor: 0.3,
            engineer: 0.1,
            qobuz_similar: 0.5,
            shared_tag: 0.2,
            user_affinity: 0.3,
        }
    }

    /// Create weights optimized for sound-alike discovery
    pub fn similarity_focused() -> Self {
        Self {
            member_of_band: 0.6,
            has_member: 0.5,
            founder: 0.5,
            collaboration: 0.7,
            performer: 0.5,
            composer: 0.4,
            producer: 0.3,
            conductor: 0.3,
            engineer: 0.2,
            qobuz_similar: 1.0, // Prioritize Qobuz similarity
            shared_tag: 0.5,
            user_affinity: 0.6,
        }
    }
}
