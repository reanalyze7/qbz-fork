//! Curated-playlist and fresh-release read types.

use serde::{Deserialize, Serialize};

/// A "Created for you" curated playlist's metadata
///
/// Returned by `GET /user/{user_name}/playlists/createdfor`. Describes one of
/// the auto-generated playlists (Weekly Jams, Weekly Exploration, Top
/// Discoveries, etc.) without its tracks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LbPlaylistMeta {
    /// MusicBrainz playlist ID (last path segment of the JSPF `identifier`)
    pub playlist_mbid: String,
    /// Playlist title
    pub title: String,
    /// Generation algorithm patch name (e.g. `weekly-jams`, `weekly-exploration`)
    pub source_patch: Option<String>,
    /// Playlist annotation / description, if available
    pub annotation: Option<String>,
    /// ISO-8601 creation timestamp, if available
    pub created_at: Option<String>,
}

/// A single track inside a curated playlist (JSPF entry)
///
/// Returned by `GET /playlist/{playlist_mbid}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LbPlaylistTrack {
    /// MusicBrainz recording ID (last path segment of the JSPF `identifier`)
    pub recording_mbid: Option<String>,
    /// Track title
    pub title: String,
    /// Artist name (JSPF `creator`)
    pub artist_name: String,
    /// Release/album name (JSPF `album`), if available
    pub release_name: Option<String>,
    /// Cover Art Archive image ID, if available
    pub caa_id: Option<i64>,
    /// Cover Art Archive release MBID, if available
    pub caa_release_mbid: Option<String>,
}

/// A personalized fresh release entry
///
/// Returned by `GET /user/{user_name}/fresh_releases`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LbFreshRelease {
    /// Release/album name
    pub release_name: String,
    /// Artist credit name
    pub artist_credit_name: String,
    /// MusicBrainz release ID, if available
    pub release_mbid: Option<String>,
    /// MusicBrainz release-group ID, if available
    pub release_group_mbid: Option<String>,
    /// Release-group primary type (Album, Single, EP, ...), if available
    pub primary_type: Option<String>,
    /// Cover Art Archive image ID, if available
    pub caa_id: Option<i64>,
    /// Cover Art Archive release MBID, if available
    pub caa_release_mbid: Option<String>,
    /// Release date (ISO-8601), if available
    pub release_date: Option<String>,
    /// Number of listens recorded for this release, if available
    pub listen_count: Option<u64>,
}
