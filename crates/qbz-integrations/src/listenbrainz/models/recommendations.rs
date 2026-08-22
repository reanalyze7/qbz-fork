//! CF-recommendation, listen-history, and recording-metadata read types.

use serde::{Deserialize, Serialize};

/// Collaborative-filtering recommendation entry
///
/// Returned by `GET /cf/recommendation/user/{user_name}/recording`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CfRecommendation {
    /// MusicBrainz recording ID of the recommended track
    pub recording_mbid: String,
    /// Recommendation score (higher = stronger match)
    pub score: f64,
    /// ISO-8601 timestamp of the user's last listen to this recording,
    /// or `None` if never listened (null/absent in the API)
    pub latest_listened_at: Option<String>,
}

/// A single listen from a user's scrobble history
///
/// Returned by `GET /user/{user_name}/listens`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LbListen {
    /// Unix timestamp when the track was listened to
    pub listened_at: i64,
    /// Artist name
    pub artist_name: String,
    /// Track name
    pub track_name: String,
    /// MusicBrainz recording ID (from `mbid_mapping`), if the listen was mapped
    pub recording_mbid: Option<String>,
}

/// Hydrated recording metadata
///
/// Returned by `GET /metadata/recording/`. Used to enrich CF recommendation
/// MBIDs with human-readable names, artist IDs, and cover art references.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LbRecordingMeta {
    /// MusicBrainz recording ID (the response object key)
    pub recording_mbid: String,
    /// Recording (track) name
    pub recording_name: String,
    /// Primary artist credit name
    pub artist_name: String,
    /// MusicBrainz artist IDs that make up the credit
    pub artist_mbids: Vec<String>,
    /// Release/album name, if available
    pub release_name: Option<String>,
    /// Cover Art Archive image ID, if available
    pub caa_id: Option<i64>,
    /// Cover Art Archive release MBID, if available
    pub caa_release_mbid: Option<String>,
}
