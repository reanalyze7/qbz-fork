/// Playback state snapshot
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct PlaybackState {
    pub is_playing: bool,
    pub position: u64,
    pub duration: u64,
    pub track_id: u64,
    pub volume: f32,
}
