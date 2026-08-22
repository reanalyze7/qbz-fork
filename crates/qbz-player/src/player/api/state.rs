use super::*;

impl Player {
    /// Get current playback state with real-time position
    pub fn get_state(&self) -> Result<PlaybackState, String> {
        Ok(PlaybackState {
            is_playing: self.state.is_playing(),
            position: self.state.current_position(),
            duration: self.state.duration(),
            track_id: self.state.current_track_id(),
            volume: self.state.volume(),
        })
    }

    /// Get playback event for emitting to frontend
    pub fn get_playback_event(&self) -> PlaybackEvent {
        let sample_rate = self.state.get_sample_rate();
        let bit_depth = self.state.get_bit_depth();
        PlaybackEvent {
            is_playing: self.state.is_playing(),
            position: self.state.current_position(),
            duration: self.state.duration(),
            track_id: self.state.current_track_id(),
            volume: self.state.volume(),
            sample_rate: if sample_rate > 0 {
                Some(sample_rate)
            } else {
                None
            },
            bit_depth: if bit_depth > 0 { Some(bit_depth) } else { None },
            shuffle: None, // Set by caller with access to queue state
            repeat: None,  // Set by caller with access to queue state
            normalization_gain: self.state.get_normalization_gain(),
            gapless_ready: self.state.is_gapless_ready(),
            gapless_next_track_id: self.state.get_gapless_next_track_id(),
            bit_perfect_mode: self.state.get_bit_perfect_mode(),
            buffer_progress: self.state.get_buffer_progress(),
        }
    }
}
