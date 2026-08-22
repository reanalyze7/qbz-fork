use super::*;

impl Player {
    /// Pause playback
    pub fn pause(&self) -> Result<(), String> {
        self.tx
            .send(AudioCommand::Pause)
            .map_err(|e| format!("Failed to send pause command: {}", e))
    }

    /// Resume playback
    pub fn resume(&self) -> Result<(), String> {
        self.tx
            .send(AudioCommand::Resume)
            .map_err(|e| format!("Failed to send resume command: {}", e))
    }

    pub fn has_loaded_audio(&self) -> bool {
        self.state.has_loaded_audio()
    }

    /// Stop playback
    pub fn stop(&self) -> Result<(), String> {
        // Supersede any in-flight play_track so a slow CMAF/legacy fetch cannot
        // restart audio after the user stopped.
        let _ = self.begin_play();
        self.tx
            .send(AudioCommand::Stop)
            .map_err(|e| format!("Failed to send stop command: {}", e))
    }

    /// Set volume (0.0 - 1.0)
    pub fn set_volume(&self, volume: f32) -> Result<(), String> {
        let clamped = volume.clamp(0.0, 1.0);

        // Skip if volume is already at this value (prevents MPRIS/PipeWire feedback loop)
        let current = self.state.volume();
        if (clamped - current).abs() < 0.001 {
            return Ok(());
        }

        self.tx
            .send(AudioCommand::SetVolume(clamped))
            .map_err(|e| format!("Failed to send volume command: {}", e))
    }

    /// Seek to position in seconds
    pub fn seek(&self, position: u64) -> Result<(), String> {
        // Clamp to duration if known
        let duration = self.state.duration();
        let clamped_position = if duration > 0 {
            position.min(duration)
        } else {
            position
        };

        self.tx
            .send(AudioCommand::Seek(clamped_position))
            .map_err(|e| format!("Failed to send seek command: {}", e))
    }

    /// Reinitialize audio device (releases and re-acquires the device)
    /// Use this when changing audio settings like exclusive mode
    pub fn reinit_device(&self, device_name: Option<String>) -> Result<(), String> {
        self.tx
            .send(AudioCommand::ReinitDevice { device_name })
            .map_err(|e| format!("Failed to send reinit command: {}", e))
    }

    /// Release the output device without reopening it. Drops the active
    /// stream — freeing an exclusive ALSA `hw:` grab and its D-Bus
    /// reservation — and un-suspends / un-forces anything QBZ parked, so
    /// PipeWire/WirePlumber can reclaim a device QBZ was holding (e.g. a DAC
    /// left invisible to other apps after bit-perfect ALSA Direct). Pair
    /// with a device re-enumeration in the UI to surface a freed or
    /// hot-plugged DAC without restarting the app.
    pub fn release_device(&self) -> Result<(), String> {
        self.tx
            .send(AudioCommand::ReleaseDevice)
            .map_err(|e| format!("Failed to send release command: {}", e))
    }

    /// Reload audio settings from fresh config (e.g., after database update)
    /// Call this before reinit_device() to ensure Player uses latest settings
    pub fn reload_settings(&self, settings: AudioSettings) -> Result<(), String> {
        if let Ok(mut current_settings) = self.audio_settings.lock() {
            *current_settings = settings;
            Ok(())
        } else {
            Err("Failed to lock audio settings".to_string())
        }
    }
}
