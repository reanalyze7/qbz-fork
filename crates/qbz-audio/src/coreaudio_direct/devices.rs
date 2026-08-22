use coreaudio::audio_unit::{macos_helpers, Scope};

use super::{transport_types, AudioDeviceID};

/// Get the default output device ID.
pub fn get_default_output_device() -> Result<AudioDeviceID, String> {
    macos_helpers::get_default_device_id(false)
        .ok_or_else(|| "No default output device found".to_string())
}

/// Get all output device IDs.
pub fn get_output_device_ids() -> Result<Vec<AudioDeviceID>, String> {
    macos_helpers::get_audio_device_ids_for_scope(Scope::Output)
        .map_err(|e| format!("Failed to enumerate output devices: {:?}", e))
}

/// Get the name of a CoreAudio device.
pub fn get_device_name(device_id: AudioDeviceID) -> Result<String, String> {
    macos_helpers::get_device_name(device_id)
        .map_err(|e| format!("Failed to get device name: {:?}", e))
}

/// Find a CoreAudio output device ID by its name.
pub fn find_device_by_name(name: &str) -> Result<Option<AudioDeviceID>, String> {
    // get_device_id_from_name: input=false means output device
    Ok(macos_helpers::get_device_id_from_name(name, false))
}

/// Resolve an optional QBZ output device name to a CoreAudio output device ID.
/// `None` means the current system default output device.
pub fn resolve_output_device_id(device_name: Option<&str>) -> Result<AudioDeviceID, String> {
    match device_name {
        Some(name) => find_device_by_name(name)?
            .ok_or_else(|| format!("CoreAudio output device '{}' not found", name)),
        None => get_default_output_device(),
    }
}

/// Resolve an optional QBZ output device name to the exact CoreAudio device name.
/// `None` means the current system default output device.
pub fn resolve_output_device_name(device_name: Option<&str>) -> Result<String, String> {
    let device_id = resolve_output_device_id(device_name)?;
    get_device_name(device_id)
}

/// Get the transport type of a device (USB, built-in, Bluetooth, etc.)
pub fn get_device_transport_type(device_id: AudioDeviceID) -> Option<String> {
    let transport = macos_helpers::get_device_transport_type(device_id).ok()?;

    let transport_str = if transport == transport_types::BUILT_IN {
        "built-in"
    } else if transport == transport_types::USB {
        "usb"
    } else if transport == transport_types::BLUETOOTH || transport == transport_types::BLUETOOTH_LE
    {
        "bluetooth"
    } else if transport == transport_types::HDMI || transport == transport_types::DISPLAY_PORT {
        "hdmi"
    } else if transport == transport_types::THUNDERBOLT {
        "thunderbolt"
    } else if transport == transport_types::FIREWIRE {
        "firewire"
    } else if transport == transport_types::VIRTUAL {
        "virtual"
    } else if transport == transport_types::AGGREGATE {
        "aggregate"
    } else {
        "unknown"
    };

    Some(transport_str.to_string())
}
