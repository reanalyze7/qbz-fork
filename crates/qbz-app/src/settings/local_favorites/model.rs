use serde::{Deserialize, Serialize};

/// Database file name, joined onto the per-user data dir by the lifecycle layer.
pub const DB_FILE_NAME: &str = "local_favorites.db";

/// A favorited local item with its display snapshot.
///
/// Ids are Strings: album = the local group key (contains `|`/`/`),
/// artist = the artist NAME (local artists have no numeric id), track =
/// `file_path` — the stable key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalFavItem {
    /// "album" | "artist" | "track".
    pub kind: String,
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub artwork_url: String,
    /// Denormalized artist name (for per-artist counts); empty for kind="artist".
    pub artist: String,
    /// "local" — never "qobuz_download".
    pub source: String,
    /// Unix seconds; the ordering key (newest first).
    pub favorited_at: i64,
}
