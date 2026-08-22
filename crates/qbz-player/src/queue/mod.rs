//! Queue management module
//!
//! Handles playback queue with:
//! - Queue manipulation (add, remove, reorder, clear)
//! - Current track tracking
//! - Shuffle mode
//! - Repeat modes (off, all, one)
//! - Play history for going back

mod internal;
mod mutate;
mod playback;
mod repeat_and_marker;
mod shuffle;
mod state_view;

#[cfg(test)]
mod tests;

use std::collections::VecDeque;
use std::sync::Mutex;

use qbz_models::{QueueTrack, RepeatMode};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum QueueMoveDirection {
    Up,
    Down,
}

/// Internal queue state - all in one struct to avoid deadlocks
pub(crate) struct InternalState {
    /// All tracks in the queue (original order)
    pub(crate) tracks: Vec<QueueTrack>,
    /// Current playback index
    pub(crate) current_index: Option<usize>,
    /// Shuffle mode enabled
    pub(crate) shuffle: bool,
    /// Shuffled indices (when shuffle is on)
    pub(crate) shuffle_order: Vec<usize>,
    /// Position in shuffle order
    pub(crate) shuffle_position: usize,
    /// Repeat mode
    pub(crate) repeat: RepeatMode,
    /// History of played track indices (for going back)
    pub(crate) history: VecDeque<usize>,
    /// Track ID to stop after (optional)
    pub(crate) stop_after_track_id: Option<u64>,
}

/// Queue manager for handling playback queue
pub struct QueueManager {
    pub(crate) state: Mutex<InternalState>,
}

impl Default for QueueManager {
    fn default() -> Self {
        Self::new()
    }
}

impl QueueManager {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(InternalState {
                tracks: Vec::new(),
                current_index: None,
                shuffle: false,
                shuffle_order: Vec::new(),
                shuffle_position: 0,
                repeat: RepeatMode::Off,
                history: VecDeque::with_capacity(50),
                stop_after_track_id: None,
            }),
        }
    }

    /// The track at the current playback index, if any.
    pub fn current(&self) -> Option<QueueTrack> {
        let state = self.state.lock().unwrap();
        state
            .current_index
            .and_then(|idx| state.tracks.get(idx).cloned())
    }
}
