use coreaudio::audio_unit::macos_helpers;

use super::AudioDeviceID;

/// Return the PID currently owning CoreAudio Hog Mode for this device.
/// CoreAudio uses -1 when no process owns the device.
pub fn get_hogging_pid(device_id: AudioDeviceID) -> Result<i32, String> {
    macos_helpers::get_hogging_pid(device_id)
        .map(|pid| pid as i32)
        .map_err(|e| format!("Failed to query CoreAudio Hog Mode owner: {:?}", e))
}

/// Enable or disable CoreAudio Hog Mode for a device.
pub fn set_hog_mode(device_id: AudioDeviceID, enabled: bool) -> Result<(), String> {
    let current_pid = get_hogging_pid(device_id)?;
    let our_pid = std::process::id() as i32;

    if enabled {
        if current_pid == our_pid {
            log::info!(
                "[CoreAudio] Hog Mode already owned by Qoqobuz for device {}",
                device_id
            );
            return Ok(());
        }
        if current_pid != -1 && current_pid != 0 {
            return Err(format!(
                "CoreAudio device {} is already hogged by pid {}",
                device_id, current_pid
            ));
        }

        let new_pid = macos_helpers::toggle_hog_mode(device_id)
            .map(|pid| pid as i32)
            .map_err(|e| format!("Failed to enable CoreAudio Hog Mode: {:?}", e))?;
        if new_pid != our_pid {
            return Err(format!(
                "CoreAudio Hog Mode was not acquired for device {} (owner pid: {})",
                device_id, new_pid
            ));
        }

        log::info!("[CoreAudio] Hog Mode acquired for device {}", device_id);
        return Ok(());
    }

    if current_pid == our_pid {
        let new_pid = macos_helpers::toggle_hog_mode(device_id)
            .map(|pid| pid as i32)
            .map_err(|e| format!("Failed to release CoreAudio Hog Mode: {:?}", e))?;
        log::info!(
            "[CoreAudio] Hog Mode released for device {} (owner pid now {})",
            device_id,
            new_pid
        );
    } else {
        log::debug!(
            "[CoreAudio] Hog Mode release skipped for device {} (owner pid: {})",
            device_id,
            current_pid
        );
    }

    Ok(())
}
