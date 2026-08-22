//! Discover index / playlists response types.

use serde::{Deserialize, Serialize};

use super::{DiscoverAlbum, PlaylistGenre, PlaylistOwner};

/// Discover index response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverResponse {
    pub containers: DiscoverContainers,
}

/// All discover containers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverContainers {
    pub playlists: Option<DiscoverContainer<DiscoverPlaylist>>,
    pub ideal_discography: Option<DiscoverContainer<DiscoverAlbum>>,
    pub playlists_tags: Option<DiscoverContainer<PlaylistTag>>,
    pub new_releases: Option<DiscoverContainer<DiscoverAlbum>>,
    pub qobuzissims: Option<DiscoverContainer<DiscoverAlbum>>,
    pub most_streamed: Option<DiscoverContainer<DiscoverAlbum>>,
    pub press_awards: Option<DiscoverContainer<DiscoverAlbum>>,
    pub album_of_the_week: Option<DiscoverContainer<DiscoverAlbum>>,
}

/// Generic discover container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverContainer<T> {
    pub id: String,
    pub data: DiscoverData<T>,
}

/// Generic discover data with items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverData<T> {
    pub has_more: bool,
    pub items: Vec<T>,
}

/// Playlist from discover endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverPlaylist {
    pub id: u64,
    pub name: String,
    pub owner: PlaylistOwner,
    pub image: DiscoverPlaylistImage,
    pub description: Option<String>,
    pub duration: u32,
    pub tracks_count: u32,
    pub genres: Option<Vec<PlaylistGenre>>,
    pub tags: Option<Vec<PlaylistTag>>,
}

/// Playlist image from discover
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverPlaylistImage {
    pub rectangle: Option<String>,
    pub covers: Option<Vec<String>>,
}

/// Playlist tag (for filtering)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistTag {
    pub id: u64,
    pub slug: String,
    pub name: String,
}

/// Raw playlist tag from /playlist/getTags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawPlaylistTag {
    pub slug: String,
    pub name_json: String,
    pub position: Option<String>,
    pub is_discover: Option<String>,
    pub featured_tag_id: Option<String>,
}

/// Response from /playlist/getTags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistTagsResponse {
    pub tags: Vec<RawPlaylistTag>,
}

/// Response from discover/playlists endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverPlaylistsResponse {
    pub has_more: bool,
    pub items: Vec<DiscoverPlaylist>,
}
