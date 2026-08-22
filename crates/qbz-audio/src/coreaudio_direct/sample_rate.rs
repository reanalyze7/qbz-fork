use coreaudio::audio_unit::macos_helpers;
use coreaudio::Error;
use objc2_core_audio::{
    kAudioDevicePropertyNominalSampleRate, kAudioObjectPropertyElementMaster,
    kAudioObjectPropertyScopeGlobal, AudioObjectGetPropertyData, AudioObjectPropertyAddress,
};
use std::{
    mem,
    ptr::{null, NonNull},
};

use super::{AudioDeviceID, COMMON_SAMPLE_RATES};

/// Query supported sample rates for a CoreAudio device.
/// Returns discrete rates from the device's available nominal sample rate ranges.
pub fn query_supported_sample_rates(device_id: AudioDeviceID) -> Result<Vec<u32>, String> {
    let ranges = macos_helpers::get_available_sample_rates(device_id)
        .map_err(|e| format!("Failed to get sample rate ranges: {:?}", e))?;

    let mut rates = Vec::new();
    for range in &ranges {
        if (range.mMinimum - range.mMaximum).abs() < 0.5 {
            // Point value (min == max)
            rates.push(range.mMinimum as u32);
        } else {
            // Continuous range — check which common rates fall within it
            for &rate in COMMON_SAMPLE_RATES {
                let rate_f = rate as f64;
                if rate_f >= range.mMinimum && rate_f <= range.mMaximum {
                    rates.push(rate);
                }
            }
        }
    }

    rates.sort_unstable();
    rates.dedup();
    Ok(rates)
}

/// Set the nominal sample rate of a device.
/// Delegates to coreaudio-rs which handles async confirmation with a 2-second timeout.
pub fn set_nominal_sample_rate(device_id: AudioDeviceID, target_rate: u32) -> Result<(), String> {
    log::info!(
        "[CoreAudio] Switching sample rate to {}Hz on device {}",
        target_rate,
        device_id
    );

    macos_helpers::set_device_sample_rate(device_id, target_rate as f64)
        .map_err(|e| format!("Failed to set sample rate to {}Hz: {:?}", target_rate, e))?;

    log::info!("[CoreAudio] Sample rate switched to {}Hz", target_rate);
    Ok(())
}

/// Get the current nominal sample rate of a CoreAudio device.
pub fn get_nominal_sample_rate(device_id: AudioDeviceID) -> Result<u32, String> {
    unsafe {
        let property_address = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyNominalSampleRate,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMaster,
        };
        let mut rate = 0.0_f64;
        let data_size = mem::size_of::<f64>() as u32;
        let status = AudioObjectGetPropertyData(
            device_id,
            NonNull::from(&property_address),
            0,
            null(),
            NonNull::from(&data_size),
            NonNull::from(&mut rate).cast(),
        );
        Error::from_os_status(status)
            .map_err(|e| format!("Failed to query CoreAudio nominal sample rate: {:?}", e))?;
        if !rate.is_finite() || rate <= 0.0 {
            return Err(format!("Invalid CoreAudio nominal sample rate: {}", rate));
        }
        Ok(rate.round() as u32)
    }
}
