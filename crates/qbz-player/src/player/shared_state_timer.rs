use super::*;

impl SharedState {
    /// Mark playback as started/resumed at current position
    pub(crate) fn start_playback_timer(&self, position: u64) {
        let now_millis = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        self.playback_start_millis
            .store(now_millis, Ordering::SeqCst);
        self.position_at_start.store(position, Ordering::SeqCst);
    }

    /// Mark playback as paused, saving current position
    pub(crate) fn pause_playback_timer(&self) {
        let current_pos = self.current_position();
        self.position.store(current_pos, Ordering::SeqCst);
        self.playback_start_millis.store(0, Ordering::SeqCst);
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing.load(Ordering::SeqCst)
    }

    pub fn position(&self) -> u64 {
        self.position.load(Ordering::SeqCst)
    }

    pub fn duration(&self) -> u64 {
        self.duration.load(Ordering::SeqCst)
    }

    pub fn current_track_id(&self) -> u64 {
        self.current_track_id.load(Ordering::SeqCst)
    }

    pub fn set_loaded_audio(&self, loaded: bool) {
        self.has_loaded_audio.store(loaded, Ordering::SeqCst);
    }

    pub fn has_loaded_audio(&self) -> bool {
        self.has_loaded_audio.load(Ordering::SeqCst)
    }

    pub fn volume(&self) -> f32 {
        f32::from_bits(self.volume.load(Ordering::SeqCst))
    }
}
