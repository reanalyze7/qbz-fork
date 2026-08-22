//! Playback Operations: thin sync wraps over `self.player`.

use std::sync::Arc;

use qbz_models::FrontendAdapter;
use qbz_player::{PlaybackState, Player};

use crate::error::CoreError;

use super::QbzCore;

impl<A: FrontendAdapter + Send + Sync + 'static> QbzCore<A> {
    /// Pause playback
    pub fn pause(&self) -> Result<(), CoreError> {
        self.player.pause().map_err(|e| CoreError::Playback(e))
    }

    /// Resume playback
    pub fn resume(&self) -> Result<(), CoreError> {
        self.player.resume().map_err(|e| CoreError::Playback(e))
    }

    /// Stop playback
    pub fn stop(&self) -> Result<(), CoreError> {
        self.player.stop().map_err(|e| CoreError::Playback(e))
    }

    /// Seek to position in seconds
    pub fn seek(&self, position: u64) -> Result<(), CoreError> {
        self.player
            .seek(position)
            .map_err(|e| CoreError::Playback(e))
    }

    /// Set volume (0.0 - 1.0)
    pub fn set_volume(&self, volume: f32) -> Result<(), CoreError> {
        self.player
            .set_volume(volume)
            .map_err(|e| CoreError::Playback(e))
    }

    /// Get current playback state
    pub fn get_playback_state(&self) -> PlaybackState {
        let state = &self.player.state;
        PlaybackState {
            is_playing: state.is_playing(),
            position: state.current_position(),
            duration: state.duration(),
            track_id: state.current_track_id(),
            volume: state.volume(),
        }
    }

    /// Get the player (for advanced usage)
    pub fn player(&self) -> Arc<Player> {
        Arc::clone(&self.player)
    }
}
