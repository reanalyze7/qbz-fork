//! Auth/connection-status types and the offline submission queue row.

use serde::{Deserialize, Serialize};

/// User info response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserInfo {
    pub user_name: String,
}

/// Token validation response
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct TokenValidationResponse {
    pub code: i32,
    pub message: String,
    pub valid: bool,
    #[serde(default)]
    pub user_name: Option<String>,
}

/// ListenBrainz connection status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListenBrainzStatus {
    pub connected: bool,
    pub user_name: Option<String>,
    pub enabled: bool,
}

/// Queued listen for offline submission
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueuedListen {
    pub id: i64,
    pub listened_at: i64,
    pub artist_name: String,
    pub track_name: String,
    pub release_name: Option<String>,
    pub recording_mbid: Option<String>,
    pub release_mbid: Option<String>,
    pub artist_mbids: Option<Vec<String>>,
    pub isrc: Option<String>,
    pub duration_ms: Option<u64>,
    pub created_at: i64,
    pub attempts: i32,
    pub sent: bool,
}
