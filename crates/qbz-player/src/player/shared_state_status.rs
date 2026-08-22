use super::*;

impl SharedState {
    /// Clearing (`error = false`) also drops any pending
    /// `stream_error_message`. This is intentional: if init recovers before
    /// the Tauri polling loop drains the message, we'd rather swallow the
    /// toast than surface a notification for a transient failure the user
    /// never perceived. The trade-off is that a fast record→clear→drain
    /// sequence loses the message — accepted because a recovered error is
    /// not a user-actionable event.
    /// 0 = none, 1 = DoP, 2 = native BE, 3 = native LE.
    pub fn set_dsd_mode(&self, mode: u8) {
        self.dsd_direct.store(mode, Ordering::SeqCst);
    }

    pub fn dsd_mode(&self) -> u8 {
        self.dsd_direct.load(Ordering::SeqCst)
    }

    pub fn is_dsd_direct(&self) -> bool {
        self.dsd_direct.load(Ordering::SeqCst) != 0
    }

    pub fn set_stream_error(&self, error: bool) {
        self.stream_error.store(error, Ordering::SeqCst);
        if !error {
            if let Ok(mut m) = self.stream_error_message.write() {
                *m = None;
            }
        }
    }

    pub fn has_stream_error(&self) -> bool {
        self.stream_error.load(Ordering::SeqCst)
    }

    /// Record a user-readable error explanation alongside `stream_error=true`.
    /// The message is drained once via `take_stream_error_message` so the UI
    /// fires the toast exactly once per error.
    pub fn record_stream_error(&self, message: impl Into<String>) {
        self.stream_error.store(true, Ordering::SeqCst);
        if let Ok(mut m) = self.stream_error_message.write() {
            *m = Some(message.into());
        }
    }

    /// Atomically take the pending stream-error message (if any). Returns
    /// `None` when no message is pending or has already been read.
    pub fn take_stream_error_message(&self) -> Option<String> {
        self.stream_error_message
            .write()
            .ok()
            .and_then(|mut m| m.take())
    }

    /// Start a new play intent; returns the generation token for this intent
    /// (see `Player::begin_play`).
    pub(crate) fn begin_play(&self) -> u64 {
        self.play_generation.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// The most recent play generation (the token a play command sent right
    /// now would carry).
    pub(crate) fn current_play_generation(&self) -> u64 {
        self.play_generation.load(Ordering::SeqCst)
    }

    /// True while `gen` is still the newest play intent. The audio thread
    /// uses this to abandon buffer waits for superseded plays (#591).
    pub(crate) fn is_current_play(&self, gen: u64) -> bool {
        self.current_play_generation() == gen
    }

    pub fn set_stream_quality(&self, sample_rate: u32, bit_depth: u32) {
        self.sample_rate.store(sample_rate, Ordering::SeqCst);
        self.bit_depth.store(bit_depth, Ordering::SeqCst);
    }

    /// Set the current bit-perfect mode for the active stream.
    /// Pass None when no stream is active (e.g., after stop).
    pub fn set_bit_perfect_mode(&self, mode: Option<BitPerfectMode>) {
        let code = match mode {
            None => 0,
            Some(BitPerfectMode::Disabled) => 1,
            Some(BitPerfectMode::DirectHardware) => 2,
            Some(BitPerfectMode::PluginFallback) => 3,
        };
        self.bit_perfect_mode.store(code, Ordering::SeqCst);
    }

    /// Get the current bit-perfect mode for the active stream.
    /// Returns None when no stream has been initialized yet.
    pub fn get_bit_perfect_mode(&self) -> Option<BitPerfectMode> {
        match self.bit_perfect_mode.load(Ordering::SeqCst) {
            0 => None,
            1 => Some(BitPerfectMode::Disabled),
            2 => Some(BitPerfectMode::DirectHardware),
            3 => Some(BitPerfectMode::PluginFallback),
            _ => None,
        }
    }

    pub fn get_sample_rate(&self) -> u32 {
        self.sample_rate.load(Ordering::SeqCst)
    }

    pub fn get_bit_depth(&self) -> u32 {
        self.bit_depth.load(Ordering::SeqCst)
    }

    /// Set the current normalization gain factor.
    /// Stores f32 as u32 bits. Pass None (or 0.0) to indicate no normalization.
    pub fn set_normalization_gain(&self, gain: Option<f32>) {
        let bits = gain.unwrap_or(0.0).to_bits();
        self.normalization_gain.store(bits, Ordering::SeqCst);
    }

    /// Get the current normalization gain factor.
    /// Returns None if normalization is not active (gain is 0.0).
    pub fn get_normalization_gain(&self) -> Option<f32> {
        let bits = self.normalization_gain.load(Ordering::SeqCst);
        let gain = f32::from_bits(bits);
        if gain == 0.0 {
            None
        } else {
            Some(gain)
        }
    }
}
