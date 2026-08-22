//! CMAF segmented streaming session/response types.

use serde::{Deserialize, Serialize};

use super::StreamRestriction;

/// Response from POST /api.json/0.2/session/start
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStartResponse {
    pub session_id: String,
    pub expires_at: u64,
    #[serde(default)]
    pub infos: Option<String>,
}

/// Response from GET /api.json/0.2/file/url (CMAF segmented streaming)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackFileUrl {
    #[serde(default)]
    pub url_template: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub mime_type: Option<String>,
    #[serde(default)]
    pub n_segments: u8,
    #[serde(default)]
    pub key_id: Option<String>,
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub sampling_rate: Option<u32>,
    #[serde(default)]
    pub bit_depth: Option<u32>,
    #[serde(default)]
    pub bits_depth: Option<u32>,
    #[serde(default)]
    pub duration: Option<f64>,
    #[serde(default)]
    pub n_samples: Option<u64>,
    #[serde(default)]
    pub format_id: Option<u32>,
    #[serde(default)]
    pub track_id: Option<u64>,
    #[serde(default)]
    pub restrictions: Vec<StreamRestriction>,
}
