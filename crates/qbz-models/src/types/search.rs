//! Search / favorites response types.

use serde::{Deserialize, Serialize};

use super::{Album, Artist, Playlist, Track};

/// Search results container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    pub albums: Option<SearchResultsPage<Album>>,
    pub tracks: Option<SearchResultsPage<Track>>,
    pub artists: Option<SearchResultsPage<Artist>>,
    pub playlists: Option<SearchResultsPage<Playlist>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchResultsPage<T> {
    #[serde(default = "Vec::new")]
    pub items: Vec<T>,
    // `/album/suggest` returns a page with only `{limit, items}` (no `total`
    // or `offset`); without defaults the whole response failed to deserialize
    // and the album "Suggestions" carousel silently never showed. Defaulting
    // the pagination scalars to 0 is harmless — only `items` is consumed there.
    #[serde(default)]
    pub total: u32,
    #[serde(default)]
    pub offset: u32,
    #[serde(default)]
    pub limit: u32,
}

/// Response from `/album/suggest` — albums similar to a seed album.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumSuggestResponse {
    #[serde(default)]
    pub algorithm: Option<String>,
    #[serde(default)]
    pub albums: Option<SearchResultsPage<Album>>,
}

/// One entry of the Qobuz `most_popular` block in a combined search.
/// Serde tagging matches the legacy `V2MostPopularItem` so the Tauri
/// command's response shape is unchanged.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content", rename_all = "lowercase")]
pub enum MostPopularItem {
    Tracks(Track),
    Albums(Album),
    Artists(Artist),
}

/// Combined search result: the four category pages plus an optional
/// "most popular" hero entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchAllResults {
    pub albums: SearchResultsPage<Album>,
    pub tracks: SearchResultsPage<Track>,
    pub artists: SearchResultsPage<Artist>,
    pub playlists: SearchResultsPage<Playlist>,
    pub most_popular: Option<MostPopularItem>,
}

/// Favorites container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Favorites {
    pub albums: Option<SearchResultsPage<Album>>,
    pub tracks: Option<SearchResultsPage<Track>>,
    pub artists: Option<SearchResultsPage<Artist>>,
}
