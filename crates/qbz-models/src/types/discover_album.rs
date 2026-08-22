//! Discover-endpoint album shape + its nested image/dates/audio-info types.

use serde::{Deserialize, Serialize};

use super::{Genre, Label};

/// Album from discover endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverAlbum {
    pub id: String,
    pub title: String,
    pub version: Option<String>,
    pub track_count: Option<u32>,
    pub duration: Option<u32>,
    pub parental_warning: Option<bool>,
    pub image: DiscoverAlbumImage,
    pub artists: Vec<DiscoverArtist>,
    pub label: Option<Label>,
    pub genre: Option<Genre>,
    pub dates: Option<DiscoverAlbumDates>,
    pub audio_info: Option<DiscoverAudioInfo>,
}

/// Album image from discover endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverAlbumImage {
    pub small: Option<String>,
    pub thumbnail: Option<String>,
    pub large: Option<String>,
}

/// Artist in discover album
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverArtist {
    pub id: u64,
    pub name: String,
    pub roles: Option<Vec<String>>,
}

/// Album dates from discover
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverAlbumDates {
    pub download: Option<String>,
    pub original: Option<String>,
    pub stream: Option<String>,
}

/// Audio info from discover album
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverAudioInfo {
    pub maximum_sampling_rate: Option<f64>,
    pub maximum_bit_depth: Option<u32>,
    pub maximum_channel_count: Option<u32>,
}
