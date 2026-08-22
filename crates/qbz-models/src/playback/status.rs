//! Current playback state + detailed status.

use serde::{Deserialize, Serialize};

/// Current playback state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybackState {
    /// No track loaded
    Stopped,
    /// Track loaded and playing
    Playing,
    /// Track loaded but paused
    Paused,
    /// Loading/buffering track
    Loading,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self::Stopped
    }
}

/// Detailed playback status with position and duration
#[derive(Debug, Clone, Serialize)]
pub struct PlaybackStatus {
    pub state: PlaybackState,
    pub track_id: Option<u64>,
    pub position_secs: u64,
    pub duration_secs: u64,
    pub volume: f32,
    /// Sample rate of currently playing track (Hz)
    pub sample_rate: Option<u32>,
    /// Bit depth of currently playing track
    pub bit_depth: Option<u32>,
}

impl Default for PlaybackStatus {
    fn default() -> Self {
        Self {
            state: PlaybackState::Stopped,
            track_id: None,
            position_secs: 0,
            duration_secs: 0,
            volume: 1.0,
            sample_rate: None,
            bit_depth: None,
        }
    }
}
