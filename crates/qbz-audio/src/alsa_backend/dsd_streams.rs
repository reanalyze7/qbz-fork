//! DoP and native-DSD ALSA direct stream creation (DSD plan phases 2-3).

use super::device_id::extract_card_name_from_device;
use super::pipewire_suspend::suspend_default_sink_for_exclusive;
use super::proc_rates::get_hw_supported_rates;
use crate::backend::AlsaDirectError;
use crate::AlsaDirectStream;

/// Create a DoP-capable ALSA direct stream (DSD plan Phase 2). ADDITIVE
/// helper next to the protected PCM path: exclusive `hw:` open, S32_LE only,
/// exact carrier rate, `/proc` rate pre-check, and the same busy backoff +
/// PipeWire-suspend dance as the PCM path. Errors are user-meaningful (the
/// caller falls back to DSD→PCM conversion with a toast).
pub fn create_dop_stream(
    device_id: &str,
    carrier_rate: u32,
    channels: u16,
) -> Result<AlsaDirectStream, String> {
    if !AlsaDirectStream::is_hw_device(device_id) {
        return Err(format!(
            "device '{}' is not a direct hardware (hw:) device",
            device_id
        ));
    }
    // Same base-id mapping as the PCM direct path (plughw is useless for DoP —
    // any conversion breaks the markers, so only the raw hw id is tried).
    let hw_device = if device_id.starts_with("plughw:") {
        device_id.replace("plughw:", "hw:")
    } else {
        device_id.to_string()
    };
    if let Some(card_name) = extract_card_name_from_device(device_id) {
        if let Some(hw_rates) = get_hw_supported_rates(&card_name) {
            if !hw_rates.contains(&carrier_rate) {
                return Err(format!(
                    "device does not support the {} Hz DoP carrier rate (supported: {:?})",
                    carrier_rate, hw_rates
                ));
            }
        }
    }
    match AlsaDirectStream::new_dop(&hw_device, carrier_rate, channels) {
        Ok(stream) => Ok(stream),
        Err(first_err) => {
            let error = AlsaDirectError::from_alsa_error(&first_err);
            if !matches!(error, AlsaDirectError::DeviceBusy(_)) {
                return Err(first_err);
            }
            log::info!("[ALSA Backend] DoP device busy — retrying with backoff");
            suspend_default_sink_for_exclusive();
            let mut last_err = first_err;
            for delay_ms in [50u64, 100, 200, 400, 800] {
                std::thread::sleep(std::time::Duration::from_millis(delay_ms));
                match AlsaDirectStream::new_dop(&hw_device, carrier_rate, channels) {
                    Ok(stream) => return Ok(stream),
                    Err(e) => last_err = e,
                }
            }
            Err(last_err)
        }
    }
}

/// Create a NATIVE-DSD ALSA direct stream (DSD plan Phase 3). Same shape as
/// [`create_dop_stream`]: hw-only, busy backoff. No /proc rate pre-check —
/// the DSD altset rates aren't reliably visible there; the open itself
/// validates (and fails cleanly when the kernel quirk is missing).
/// Returns the stream plus `little_endian` for the packer.
pub fn create_native_dsd_stream(
    device_id: &str,
    dsd_rate: u32,
    channels: u16,
) -> Result<(AlsaDirectStream, bool), String> {
    if !AlsaDirectStream::is_hw_device(device_id) {
        return Err(format!(
            "device '{}' is not a direct hardware (hw:) device",
            device_id
        ));
    }
    let hw_device = if device_id.starts_with("plughw:") {
        device_id.replace("plughw:", "hw:")
    } else {
        device_id.to_string()
    };
    match AlsaDirectStream::new_native_dsd(&hw_device, dsd_rate, channels) {
        Ok(pair) => Ok(pair),
        Err(first_err) => {
            let error = AlsaDirectError::from_alsa_error(&first_err);
            if !matches!(error, AlsaDirectError::DeviceBusy(_)) {
                return Err(first_err);
            }
            log::info!("[ALSA Backend] Native-DSD device busy — retrying with backoff");
            suspend_default_sink_for_exclusive();
            let mut last_err = first_err;
            for delay_ms in [50u64, 100, 200, 400, 800] {
                std::thread::sleep(std::time::Duration::from_millis(delay_ms));
                match AlsaDirectStream::new_native_dsd(&hw_device, dsd_rate, channels) {
                    Ok(pair) => return Ok(pair),
                    Err(e) => last_err = e,
                }
            }
            Err(last_err)
        }
    }
}
