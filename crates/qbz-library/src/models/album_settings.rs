//! Per-album settings for local library albums

use serde::{Deserialize, Serialize};

/// Album settings for local library albums
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumSettings {
    pub album_group_key: String,
    pub hidden: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

impl AlbumSettings {
    pub fn new(album_group_key: String) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        Self {
            album_group_key,
            hidden: false,
            created_at: now,
            updated_at: now,
        }
    }
}
