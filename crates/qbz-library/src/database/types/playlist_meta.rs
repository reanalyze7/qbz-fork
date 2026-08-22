//! Playlist statistics and playlist-folder-organization types.

/// Playlist statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlaylistStats {
    pub qobuz_playlist_id: u64,
    pub play_count: u32,
    pub last_played_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Playlist folder for organizing playlists locally
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlaylistFolder {
    pub id: String,
    pub name: String,
    pub icon_type: String,   // "preset" or "custom"
    pub icon_preset: String, // lucide icon name
    pub icon_color: String,  // hex color
    pub custom_image_path: Option<String>,
    pub is_hidden: bool,
    pub position: i32,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Default for PlaylistStats {
    fn default() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        Self {
            qobuz_playlist_id: 0,
            play_count: 0,
            last_played_at: None,
            created_at: now,
            updated_at: now,
        }
    }
}
