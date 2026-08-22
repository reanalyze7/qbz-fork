//! Track model + the album summary embedded in track responses.

use serde::{Deserialize, Serialize};

use super::{Artist, Genre, ImageSet, Label};

/// Track model
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Track {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub title: String,
    /// Subtitle/edition info from Qobuz (e.g. "Player's Ball Mix",
    /// "Nine Inch Noize Version", "Remastered 2024"). Frontend renders
    /// it parenthesized after the title so remix and reissue albums are
    /// distinguishable from originals (issue #360).
    pub version: Option<String>,
    /// Classical "work" the track belongs to (e.g. "Symphony No. 9 in D minor,
    /// Op. 125"). Qobuz returns it on the track object (always present in the
    /// envelope, `null` for non-classical catalog). Drives the per-work section
    /// headers on the album view, mirroring the official Qobuz player (PR #536).
    pub work: Option<String>,
    pub isrc: Option<String>,
    #[serde(default)]
    pub duration: u32,
    #[serde(default)]
    pub track_number: u32,
    pub media_number: Option<u32>,
    pub performer: Option<Artist>,
    pub album: Option<AlbumSummary>,
    #[serde(default)]
    pub hires: bool,
    #[serde(default)]
    pub hires_streamable: bool,
    pub maximum_sampling_rate: Option<f64>,
    pub maximum_bit_depth: Option<u32>,
    #[serde(default)]
    pub streamable: bool,
    #[serde(default)]
    pub parental_warning: bool,
    /// Playlist-specific: ID within the playlist (for removal)
    pub playlist_track_id: Option<u64>,
    /// Performers/credits string (format: "Name, Role - Name, Role")
    pub performers: Option<String>,
    /// Composer information
    pub composer: Option<Artist>,
    /// Copyright information
    pub copyright: Option<String>,
}

/// Album summary (embedded in track responses)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumSummary {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub image: ImageSet,
    /// Label (if returned in track response)
    pub label: Option<Label>,
    /// Genre (when returned, e.g. on favorites track album objects).
    pub genre: Option<Genre>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracksContainer {
    pub items: Vec<Track>,
    pub total: u32,
}
