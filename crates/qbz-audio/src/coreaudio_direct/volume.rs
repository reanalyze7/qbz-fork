use objc2_core_audio::{
    kAudioDevicePropertyVolumeScalar, kAudioObjectPropertyElementMaster,
    kAudioObjectPropertyScopeOutput, AudioObjectGetPropertyData, AudioObjectHasProperty,
    AudioObjectPropertyAddress, AudioObjectSetPropertyData,
};
use std::{mem, ptr::null, ptr::NonNull};

use super::AudioDeviceID;

/// Get a CoreAudio device's current hardware output volume as scalar 0.0..1.0.
/// Tries master output first, then common stereo channel elements.
pub fn get_hardware_volume(device_id: AudioDeviceID) -> Result<f32, String> {
    for element in [kAudioObjectPropertyElementMaster, 1, 2] {
        let property_address = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyVolumeScalar,
            mScope: kAudioObjectPropertyScopeOutput,
            mElement: element,
        };

        let has_property =
            unsafe { AudioObjectHasProperty(device_id, NonNull::from(&property_address)) };
        if !has_property {
            continue;
        }

        let mut value = 0.0_f32;
        let data_size = mem::size_of::<f32>() as u32;
        let status = unsafe {
            AudioObjectGetPropertyData(
                device_id,
                NonNull::from(&property_address),
                0,
                null(),
                NonNull::from(&data_size),
                NonNull::from(&mut value).cast(),
            )
        };

        if status == 0 {
            return Ok(value.clamp(0.0, 1.0));
        }
    }

    Err(format!(
        "CoreAudio device {} does not expose readable output hardware volume",
        device_id
    ))
}

/// Set a CoreAudio device's hardware output volume using scalar 0.0..1.0.
/// Tries master output first, then common stereo channel elements.
pub fn set_hardware_volume(device_id: AudioDeviceID, volume: f32) -> Result<(), String> {
    let clamped = volume.clamp(0.0, 1.0);
    let mut last_error = None;
    let mut channel_success = false;

    for element in [kAudioObjectPropertyElementMaster, 1, 2] {
        let property_address = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyVolumeScalar,
            mScope: kAudioObjectPropertyScopeOutput,
            mElement: element,
        };

        let has_property =
            unsafe { AudioObjectHasProperty(device_id, NonNull::from(&property_address)) };
        if !has_property {
            continue;
        }

        let mut value = clamped;
        let status = unsafe {
            AudioObjectSetPropertyData(
                device_id,
                NonNull::from(&property_address),
                0,
                null(),
                mem::size_of::<f32>() as u32,
                NonNull::new((&mut value as *mut f32).cast()).expect("volume pointer"),
            )
        };

        if status == 0 {
            log::debug!(
                "[CoreAudio] Set hardware volume for device {} element {} to {:.0}%",
                device_id,
                element,
                clamped * 100.0
            );
            if element == kAudioObjectPropertyElementMaster {
                return Ok(());
            }
            channel_success = true;
            continue;
        }

        last_error = Some(status);
    }

    if channel_success {
        return Ok(());
    }

    Err(format!(
        "CoreAudio device {} does not expose settable output hardware volume{}",
        device_id,
        last_error
            .map(|status| format!(" (last OSStatus {})", status))
            .unwrap_or_default()
    ))
}
