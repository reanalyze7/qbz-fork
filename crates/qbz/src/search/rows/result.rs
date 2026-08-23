//! Plain (`Send`) row types for the search results page.

/// An album result row, before it becomes a Slint `AlbumCardItem`.
#[derive(Debug, Clone, PartialEq)]
pub struct AlbumRow {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub artist_id: String,
    pub genre: String,
    pub year: String,
    pub quality_tier: String,
    pub quality_label: String,
    pub artwork_url: String,
}

/// A track result row, before it becomes a Slint `TrackItem`.
#[derive(Debug, Clone, PartialEq)]
pub struct TrackRowData {
    pub id: String,
    pub title: String,
    pub artist: String,
    /// Performer id for the clickable artist link ("" = plain text).
    pub artist_id: String,
    /// Album id for the clickable album link ("" = plain text).
    pub album_id: String,
    pub duration: String,
    pub quality_tier: String,
    /// Detailed quality label, e.g. "Hi-Res 24-bit / 192 kHz". Used by the
    /// most-popular track hero (shown as text instead of an icon badge).
    pub quality_label: String,
    /// Exact bit-depth / sample-rate line, e.g. "24-bit / 192 kHz" — feeds the
    /// track-row quality badge (no tier prefix, unlike `quality_label`).
    pub quality_detail: String,
    pub explicit: bool,
    pub artwork_url: String,
}

/// An artist result row, before it becomes a Slint `SlimItem`.
#[derive(Debug, Clone, PartialEq)]
pub struct ArtistRow {
    pub id: String,
    pub name: String,
    pub subtitle: String,
    pub artwork_url: String,
    /// Whether the user already follows (favorites) this artist.
    pub following: bool,
}

/// A playlist result row, before it becomes a Slint `SearchPlaylistItem`.
#[derive(Debug, Clone, PartialEq)]
pub struct PlaylistRow {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    /// Up to four distinct cover URLs for the collage.
    pub cover_urls: Vec<String>,
    /// Ownership signals for the card overlay/menu (owned → favorite; foreign
    /// Qobuz → follow + copy). `is_owned` is authoritative (owner.id ==
    /// current user); `is_following`/`is_copied` are best-effort per source
    /// (favorites seeds `is_following` from the followed split; other list
    /// surfaces leave them false — the action still works id-scoped).
    pub is_owned: bool,
    pub is_following: bool,
    pub is_copied: bool,
}

/// The most-popular hero entry.
#[derive(Debug, Clone, PartialEq)]
pub enum MostPopularRow {
    None,
    Album(AlbumRow),
    Artist(ArtistRow),
    Track(TrackRowData),
}

/// The full result of a combined search, as plain `Send` data.
pub struct SearchData {
    pub query: String,
    pub albums: Vec<AlbumRow>,
    pub tracks: Vec<TrackRowData>,
    pub artists: Vec<ArtistRow>,
    pub playlists: Vec<PlaylistRow>,
    pub albums_total: u32,
    pub tracks_total: u32,
    pub artists_total: u32,
    pub playlists_total: u32,
    pub most_popular: MostPopularRow,
}
