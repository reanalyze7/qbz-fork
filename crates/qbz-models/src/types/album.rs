//! Album model + its contributor/goody sub-types.

use serde::{Deserialize, Serialize};

use super::{Artist, DiscoverAlbumDates, DiscoverAudioInfo, Genre, ImageSet, Label, TracksContainer};

/// Album model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Album {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub artist: Artist,
    #[serde(default)]
    pub image: ImageSet,
    pub release_date_original: Option<String>,
    /// Date the album becomes available for streaming (ISO YYYY-MM-DD).
    /// When in the future, the album is upcoming and cannot be fetched
    /// via `get_album` yet — Release Watch uses this to gate clicks.
    pub release_date_stream: Option<String>,
    /// Whether the album is currently streamable. False for upcoming
    /// releases, regional restrictions, or label takedowns.
    #[serde(default)]
    pub streamable: Option<bool>,
    pub label: Option<Label>,
    pub genre: Option<Genre>,
    pub tracks_count: Option<u32>,
    pub duration: Option<u32>,
    #[serde(default)]
    pub hires: bool,
    #[serde(default)]
    pub hires_streamable: bool,
    pub maximum_sampling_rate: Option<f64>,
    pub maximum_bit_depth: Option<u32>,
    /// V2 nested quality block. The modern album shape returned by
    /// `/label/getAlbums` (DiscographyAlbumDto) and `/discover`-style items
    /// nests quality here; preferred over the flat `maximum_*` fields.
    #[serde(default)]
    pub audio_info: Option<DiscoverAudioInfo>,
    /// V2 nested release dates (`{original, download, stream}`); preferred
    /// over the flat `release_date_original` when present.
    #[serde(default)]
    pub dates: Option<DiscoverAlbumDates>,
    /// The V2 wire spells the album track count `track_count` (no trailing
    /// `s`); the flat shape uses `tracks_count`.
    #[serde(default)]
    pub track_count: Option<u32>,
    /// Explicit release type when provided ("album" | "ep" | "single" |
    /// "live" | "compilation" | ...).
    #[serde(default)]
    pub release_type: Option<String>,
    #[serde(default)]
    pub tracks: Option<TracksContainer>,
    /// Universal Product Code for the album
    pub upc: Option<String>,
    /// Editorial description/review of the album
    pub description: Option<String>,
    /// Album goodies (booklets, liner notes PDFs)
    #[serde(default)]
    pub goodies: Option<Vec<Goody>>,
    /// Parental advisory / explicit content marker.
    #[serde(default)]
    pub parental_warning: Option<bool>,
    /// Full artist contributor list including roles. The primary artist is
    /// duplicated here as `roles: ["main-artist"]`; non-main entries are
    /// the album's featured artists.
    #[serde(default)]
    pub artists: Option<Vec<AlbumArtist>>,
    /// Release variant label ("2009 Remaster", "Hi-Res", "Deluxe Edition", …).
    /// Qobuz keeps this out of `title`; the web player appends it in parens so
    /// re-editions of the same album are distinguishable. Surfaced the same way
    /// on every album title (see `format_album_title`).
    #[serde(default)]
    pub version: Option<String>,
    /// Album-level composer credit (single Artist). The official web player
    /// renders this — NOT the per-track `composer` — as the "… • X
    /// (composer)" tail of the header credit line, and suppresses it when the
    /// name is the "Various Composers" placeholder. See `album::build_credits`.
    #[serde(default)]
    pub composer: Option<Artist>,
}

/// Album artist contributor entry (main artist + featured artists).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumArtist {
    pub id: u64,
    pub name: String,
    #[serde(default)]
    pub roles: Option<Vec<String>>,
}

/// A downloadable extra bundled with an album (e.g. PDF booklet)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goody {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub url: String,
    /// Original (full-size) URL
    #[serde(default)]
    pub original_url: String,
    /// File format id (e.g. 21 for PDF)
    #[serde(default)]
    pub file_format_id: Option<u32>,
    #[serde(default)]
    pub description: Option<String>,
}
