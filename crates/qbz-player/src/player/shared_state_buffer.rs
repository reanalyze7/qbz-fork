use super::*;

impl SharedState {
    /// Set streaming buffer progress (0.0 to 1.0). Pass 0.0 when not streaming.
    pub fn set_buffer_progress(&self, progress: f32) {
        self.buffer_progress
            .store(progress.to_bits(), Ordering::SeqCst);
    }

    /// Get streaming buffer progress (0.0 to 1.0). Returns None if not streaming.
    pub fn get_buffer_progress(&self) -> Option<f32> {
        let bits = self.buffer_progress.load(Ordering::SeqCst);
        let progress = f32::from_bits(bits);
        if progress <= 0.0 || progress >= 1.0 {
            None
        } else {
            Some(progress)
        }
    }

    pub fn set_current_device(&self, device: Option<String>) {
        if let Ok(mut d) = self.current_device.write() {
            *d = device;
        }
    }

    pub fn current_device(&self) -> Option<String> {
        self.current_device.read().ok().and_then(|d| d.clone())
    }

    pub fn set_gapless_ready(&self, ready: bool) {
        self.gapless_ready.store(ready, Ordering::SeqCst);
    }

    pub fn is_gapless_ready(&self) -> bool {
        self.gapless_ready.load(Ordering::SeqCst)
    }

    pub fn set_gapless_next_track_id(&self, track_id: u64) {
        self.gapless_next_track_id.store(track_id, Ordering::SeqCst);
    }

    pub fn get_gapless_next_track_id(&self) -> u64 {
        self.gapless_next_track_id.load(Ordering::SeqCst)
    }

    /// Get current position based on elapsed time since playback started
    pub fn current_position(&self) -> u64 {
        if !self.is_playing.load(Ordering::SeqCst) {
            return self.position.load(Ordering::SeqCst);
        }

        let start_millis = self.playback_start_millis.load(Ordering::SeqCst);
        if start_millis == 0 {
            return self.position.load(Ordering::SeqCst);
        }

        let now_millis = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let elapsed_secs = (now_millis.saturating_sub(start_millis)) / 1000;
        let position_at_start = self.position_at_start.load(Ordering::SeqCst);
        let duration = self.duration.load(Ordering::SeqCst);

        // Clamp to duration
        (position_at_start + elapsed_secs).min(duration)
    }

    /// Millisecond-precision companion to [`Self::current_position`] — the
    /// exact same derivation WITHOUT the whole-second truncation. READ-ONLY
    /// state derivation from the existing anchors (`playback_start_millis`
    /// is already epoch-ms; `position_at_start`/`position`/`duration` are
    /// seconds): no stream, seek, format or device path is touched.
    ///
    /// Added for the lyrics sync engine (karaoke needs sub-second
    /// resolution); semantics mirror `current_position` line by line:
    /// paused / no anchor → stored coarse position ×1000; playing →
    /// `position_at_start*1000 + (now_ms - start_millis)`, clamped to
    /// `duration*1000`.
    pub fn current_position_ms(&self) -> u64 {
        if !self.is_playing.load(Ordering::SeqCst) {
            return self.position.load(Ordering::SeqCst).saturating_mul(1000);
        }

        let start_millis = self.playback_start_millis.load(Ordering::SeqCst);
        if start_millis == 0 {
            return self.position.load(Ordering::SeqCst).saturating_mul(1000);
        }

        let now_millis = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let elapsed_ms = now_millis.saturating_sub(start_millis);
        let position_at_start_ms = self
            .position_at_start
            .load(Ordering::SeqCst)
            .saturating_mul(1000);
        let duration_ms = self.duration.load(Ordering::SeqCst).saturating_mul(1000);

        // Clamp to duration (same rule as current_position)
        position_at_start_ms.saturating_add(elapsed_ms).min(duration_ms)
    }
}
