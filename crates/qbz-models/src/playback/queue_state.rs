//! Queue state snapshot + repeat mode.

use serde::{Deserialize, Serialize};

use super::QueueTrack;

/// Repeat mode options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepeatMode {
    Off,
    All,
    One,
}

impl Default for RepeatMode {
    fn default() -> Self {
        Self::Off
    }
}

/// Queue state snapshot for frontend
#[derive(Debug, Clone, Serialize)]
pub struct QueueState {
    pub current_track: Option<QueueTrack>,
    pub current_index: Option<usize>,
    pub upcoming: Vec<QueueTrack>,
    pub history: Vec<QueueTrack>,
    pub shuffle: bool,
    pub repeat: RepeatMode,
    pub total_tracks: usize,
    pub stop_after_track_id: Option<u64>,
}
