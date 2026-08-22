//! `AlsaDirectStream::is_hw_device` and the small getters
//! (`sample_rate`/`channels`/`device_id`). Present on the Linux `impl` block
//! only — the non-Linux stub has its own copies in `stub.rs`.

use super::AlsaDirectStream;

impl AlsaDirectStream {
    /// Get sample rate
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Get channels
    pub fn channels(&self) -> u16 {
        self.channels
    }

    /// Get device ID
    pub fn device_id(&self) -> &str {
        &self.device_id
    }

    /// Check if device is a bit-perfect hardware device
    /// Includes: hw:X,Y, plughw:X,Y, and front:CARD=X,DEV=Y
    pub fn is_hw_device(device_id: &str) -> bool {
        device_id.starts_with("hw:")
            || device_id.starts_with("plughw:")
            || device_id.starts_with("front:CARD=")
    }
}
