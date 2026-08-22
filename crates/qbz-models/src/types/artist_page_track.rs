//! `/artist/page` track-family types (`PageArtistTrack` and friends).

use serde::{Deserialize, Serialize};

use super::{DiscoverAudioInfo, Genre, ImageSet, Label, PageArtistReleaseArtist, PageArtistRights};

/// Track from /artist/page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageArtistTrack {
    pub id: u64,
    pub title: String,
    pub version: Option<String>,
    pub duration: Option<u32>,
    pub isrc: Option<String>,
    pub parental_warning: Option<bool>,
    pub artist: Option<PageArtistReleaseArtist>,
    pub composer: Option<serde_json::Value>,
    pub audio_info: Option<DiscoverAudioInfo>,
    pub rights: Option<PageArtistRights>,
    pub physical_support: Option<PageArtistPhysicalSupport>,
    pub album: Option<PageArtistTrackAlbum>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageArtistPhysicalSupport {
    pub media_number: Option<u32>,
    pub track_number: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageArtistTrackAlbum {
    pub id: String,
    pub title: String,
    pub version: Option<String>,
    pub image: Option<ImageSet>,
    pub label: Option<Label>,
    pub genre: Option<Genre>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageArtistPlaylists {
    pub has_more: bool,
    pub items: Vec<PageArtistPlaylist>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageArtistPlaylist {
    pub id: u64,
    pub title: Option<String>,
    pub description: Option<String>,
    pub owner: Option<PageArtistPlaylistOwner>,
    pub tracks_count: Option<u32>,
    pub duration: Option<u32>,
    pub images: Option<PageArtistPlaylistImages>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageArtistPlaylistOwner {
    pub id: u64,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageArtistPlaylistImages {
    pub rectangle: Option<Vec<String>>,
}
