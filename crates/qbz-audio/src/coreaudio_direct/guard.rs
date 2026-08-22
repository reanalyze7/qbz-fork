use super::hog_mode::set_hog_mode;
use super::volume::{get_hardware_volume, set_hardware_volume};
use super::AudioDeviceID;

/// RAII owner for CoreAudio Hog Mode.
///
/// Captures the device's hardware volume on acquire and restores it
/// when released, so leaving Exclusive Mode returns the device to the
/// volume the user had set before QBZ took over.
#[derive(Debug)]
pub struct CoreAudioExclusiveGuard {
    device_id: AudioDeviceID,
    active: bool,
    original_hardware_volume: Option<f32>,
}

impl CoreAudioExclusiveGuard {
    /// Acquire CoreAudio Hog Mode for the given device.
    ///
    /// The guard is constructed *before* the FFI call so that any
    /// partial-acquire failure (e.g. CoreAudio transfers ownership to
    /// us but the readback fails) still triggers `Drop`, which calls
    /// `set_hog_mode(false)`. That release is a no-op when we don't
    /// actually own the device, so it's safe in either outcome and
    /// avoids leaving the device hogged on error.
    pub fn acquire(device_id: AudioDeviceID) -> Result<Self, String> {
        // Snapshot the current hardware volume before we touch anything,
        // so the user's pre-Exclusive level can be restored on release.
        // Devices without a readable volume property (knob-only DACs)
        // simply don't get a snapshot — restoration is best-effort.
        let original_hardware_volume = get_hardware_volume(device_id).ok();
        let guard = Self {
            device_id,
            active: true,
            original_hardware_volume,
        };
        set_hog_mode(device_id, true)?;
        Ok(guard)
    }

    pub fn release(&mut self) -> Result<(), String> {
        if !self.active {
            return Ok(());
        }

        // Restore the hardware volume *before* releasing Hog Mode, while
        // we still own the device. After release any other process can
        // change the volume, so doing it before keeps our restoration
        // authoritative.
        if let Some(original) = self.original_hardware_volume.take() {
            if let Err(e) = set_hardware_volume(self.device_id, original) {
                log::warn!(
                    "[CoreAudio] Failed to restore hardware volume on release: {}",
                    e
                );
            }
        }

        set_hog_mode(self.device_id, false)?;
        self.active = false;
        Ok(())
    }

    pub fn set_hardware_volume(&self, volume: f32) -> Result<(), String> {
        set_hardware_volume(self.device_id, volume)
    }
}

impl Drop for CoreAudioExclusiveGuard {
    fn drop(&mut self) {
        if let Err(e) = self.release() {
            log::warn!("[CoreAudio] Failed to release Hog Mode on drop: {}", e);
        }
    }
}
