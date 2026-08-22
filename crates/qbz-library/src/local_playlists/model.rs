//! Local-playlist types: ids, source enum, and the two row structs.

/// Id prefix that marks a local playlist. A `local:<uuid>` id can never
/// parse as the `u64` every Qobuz playlist endpoint takes.
pub const LOCAL_PLAYLIST_PREFIX: &str = "local:";

/// True when `id` names a local playlist (`local:<uuid>` namespace).
pub fn is_local_playlist_id(id: &str) -> bool {
    id.starts_with(LOCAL_PLAYLIST_PREFIX)
}

/// Track source inside a local playlist.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalPlaylistTrackSource {
    Qobuz,
    Local,
}

impl LocalPlaylistTrackSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Qobuz => "qobuz",
            Self::Local => "local",
        }
    }

    pub(crate) fn parse(s: &str) -> Self {
        match s {
            "local" => Self::Local,
            _ => Self::Qobuz,
        }
    }
}

/// One playlist row (header metadata + per-source counts).
#[derive(Debug, Clone)]
pub struct LocalPlaylist {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    /// D8: never offered for upload, never reaches any Qobuz call or
    /// QConnect queue push.
    pub offline_only: bool,
    /// B3: manager organization flags — the local twin of
    /// `playlist_settings.is_favorite` / `.hidden` (those tables are
    /// keyed by the Qobuz u64 id, unrepresentable for `local:` ids).
    pub favorite: bool,
    /// B3: hidden playlists drop from the sidebar and group under the
    /// manager's "hidden" filter.
    pub hidden: bool,
    pub custom_artwork_path: Option<String>,
    /// Sidebar folder membership (shared `playlist_folders.id`); None = root.
    pub folder_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub track_count: u32,
    pub qobuz_count: u32,
    pub local_count: u32,
}

/// One membership row, ordered by `position`.
#[derive(Debug, Clone)]
pub struct LocalPlaylistTrack {
    pub playlist_id: String,
    pub position: i32,
    pub source: LocalPlaylistTrackSource,
    pub qobuz_track_id: Option<u64>,
    pub local_path: Option<String>,
    pub added_at: i64,
}

/// Input for `add_tracks` — exactly one of the two refs per source.
#[derive(Debug, Clone)]
pub enum LocalPlaylistTrackInput {
    Qobuz(u64),
    Local(String),
}

pub(crate) fn now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}
