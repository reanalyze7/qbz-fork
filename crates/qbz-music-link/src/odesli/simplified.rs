//! Consumer-facing simplified Odesli response + content-type enum.

use std::collections::HashMap;

use serde::Serialize;

/// Simplified response for consumption
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SongLinkResponse {
    /// The main song.link URL to share
    pub page_url: String,

    /// Title of the content (if available)
    pub title: Option<String>,

    /// Artist name (if available)
    pub artist: Option<String>,

    /// Thumbnail URL (if available)
    pub thumbnail_url: Option<String>,

    /// Map of platform names to their direct URLs
    pub platforms: HashMap<String, String>,

    /// The identifier used (ISRC or UPC)
    pub identifier: String,

    /// Type of content: "track" or "album"
    pub content_type: String,
}

/// Content type for sharing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    Track,
    Album,
}

impl ContentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentType::Track => "track",
            ContentType::Album => "album",
        }
    }
}
