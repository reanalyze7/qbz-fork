//! Playlist local-settings type and its local-content status enum.

/// Playlist local settings (enhances remote Qobuz playlists)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlaylistSettings {
    pub qobuz_playlist_id: u64,
    pub custom_artwork_path: Option<String>,
    pub sort_by: String,
    pub sort_order: String,
    pub last_search_query: Option<String>,
    pub notes: Option<String>,
    pub hidden: bool,
    pub position: i32,
    pub has_local_content: LocalContentStatus,
    pub is_favorite: bool,
    pub folder_id: Option<String>, // ID of the folder this playlist belongs to (null = root)
    pub created_at: i64,
    pub updated_at: i64,
}

/// Status of local content availability for a playlist
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalContentStatus {
    Unknown,
    No,
    SomeLocal,
    AllLocal,
}

impl Default for LocalContentStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

impl LocalContentStatus {
    pub fn from_str(s: &str) -> Self {
        match s {
            "no" => Self::No,
            "some_local" => Self::SomeLocal,
            "all_local" => Self::AllLocal,
            _ => Self::Unknown,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::No => "no",
            Self::SomeLocal => "some_local",
            Self::AllLocal => "all_local",
        }
    }
}

impl Default for PlaylistSettings {
    fn default() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        Self {
            qobuz_playlist_id: 0,
            custom_artwork_path: None,
            sort_by: "default".to_string(),
            sort_order: "asc".to_string(),
            last_search_query: None,
            notes: None,
            hidden: false,
            position: 0,
            has_local_content: LocalContentStatus::Unknown,
            is_favorite: false,
            folder_id: None,
            created_at: now,
            updated_at: now,
        }
    }
}
