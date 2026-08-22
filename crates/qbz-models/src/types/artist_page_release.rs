//! `/artist/page` release-family types (`PageArtistRelease` and friends).

use serde::{Deserialize, Serialize};

use super::{DiscoverAlbumDates, DiscoverAudioInfo, Genre, ImageSet, Label, PageArtistName};

/// A group of releases by type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageArtistReleaseGroup {
    #[serde(rename = "type")]
    pub release_type: String,
    pub has_more: bool,
    pub items: Vec<PageArtistRelease>,
}

/// A release item from /artist/page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageArtistRelease {
    pub id: String,
    pub title: String,
    pub version: Option<String>,
    pub tracks_count: Option<u32>,
    pub artist: Option<PageArtistReleaseArtist>,
    pub artists: Option<Vec<PageArtistReleaseContributor>>,
    pub image: Option<ImageSet>,
    pub label: Option<Label>,
    pub genre: Option<Genre>,
    pub release_type: Option<String>,
    pub release_tags: Option<Vec<String>>,
    pub duration: Option<u32>,
    pub dates: Option<DiscoverAlbumDates>,
    pub parental_warning: Option<bool>,
    pub audio_info: Option<DiscoverAudioInfo>,
    pub rights: Option<PageArtistRights>,
    pub awards: Option<Vec<PageArtistAward>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageArtistReleaseArtist {
    pub id: u64,
    pub name: PageArtistName,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageArtistReleaseContributor {
    pub id: u64,
    pub name: String,
    pub roles: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageArtistRights {
    pub streamable: Option<bool>,
    pub hires_streamable: Option<bool>,
    pub hires_purchasable: Option<bool>,
    pub purchasable: Option<bool>,
    pub downloadable: Option<bool>,
    pub previewable: Option<bool>,
    pub sampleable: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageArtistAward {
    pub id: u64,
    pub name: String,
    pub awarded_at: Option<String>,
}

/// Response from /artist/getReleasesGrid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleasesGridResponse {
    pub has_more: bool,
    pub items: Vec<PageArtistRelease>,
}
