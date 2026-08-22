//! Non-Linux stub `impl AlsaDirectStream` — all methods return "Linux only"
//! errors or defaults so the crate still compiles off-Linux.

use super::AlsaDirectStream;

#[cfg(not(target_os = "linux"))]
impl AlsaDirectStream {
    pub fn new(_device_id: &str, _sample_rate: u32, _channels: u16) -> Result<Self, String> {
        Err("ALSA Direct is only available on Linux".to_string())
    }

    pub fn write(&self, _samples: &[i16]) -> Result<(), String> {
        Err("ALSA Direct is only available on Linux".to_string())
    }

    pub fn write_f32(&self, _samples: &[f32]) -> Result<(), String> {
        Err("ALSA Direct is only available on Linux".to_string())
    }

    pub fn drain(&self) -> Result<(), String> {
        Ok(())
    }

    pub fn stop(&self) -> Result<(), String> {
        Ok(())
    }

    pub fn sample_rate(&self) -> u32 {
        44100
    }

    pub fn channels(&self) -> u16 {
        2
    }

    /// Check if device is a bit-perfect hardware device (always false on non-Linux)
    pub fn is_hw_device(_device_id: &str) -> bool {
        false
    }
}
