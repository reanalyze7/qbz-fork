/// One snapshot header row.
#[derive(Debug, Clone)]
pub struct SnapshotHeader {
    pub qobuz_playlist_id: u64,
    pub name: String,
    pub owner: Option<String>,
    /// The playlist's TOTAL Qobuz track count at snapshot time (not the
    /// offline-playable subset).
    pub track_count: Option<u32>,
    /// Unix ms when this header was last written.
    pub snapped_at: i64,
}

/// Names-producer input (one listed playlist).
#[derive(Debug, Clone)]
pub struct SnapshotNameEntry {
    pub qobuz_playlist_id: u64,
    pub name: String,
    pub owner: Option<String>,
    pub track_count: Option<u32>,
}
