//! Hardware-parameter negotiation helper for [`super::AlsaDirectStream::new`]
//! (access/format-priority-loop/channels/rate/buffer/period).

use crate::alsa_direct::recovery::ensure_exact_rate;
use alsa::pcm::{Access, Format, HwParams, PCM};
use alsa::ValueOr;

/// Negotiate hwparams on `pcm` for the standard (auto-format) PCM path and
/// apply them. Returns the format ALSA accepted.
pub(super) fn negotiate(pcm: &PCM, sample_rate: u32, channels: u16) -> Result<Format, String> {
    let hwp = HwParams::any(pcm).map_err(|e| format!("Failed to get hardware params: {}", e))?;

    // Set access type (interleaved)
    hwp.set_access(Access::RWInterleaved)
        .map_err(|e| format!("Failed to set access: {}", e))?;

    // Try formats in order of preference for bit-perfect playback
    // S24_3LE first: required by SMSL-class USB DACs (TAS1020B chip)
    // Then descending bit-depth for quality
    let format_priority = [
        (Format::S243LE, "S24_3LE"), // 24-bit packed (SMSL, Topping, Fosi DACs)
        (Format::S32LE, "S32LE"),    // 32-bit
        (Format::S24LE, "S24LE"),    // 24-bit in 32-bit container
        (Format::S16LE, "S16LE"),    // 16-bit
        (Format::FloatLE, "Float32LE"), // Float (compatibility)
    ];

    let mut selected_format = None;
    for (format, name) in &format_priority {
        if hwp.set_format(*format).is_ok() {
            log::info!("[ALSA Direct] Selected format: {}", name);
            selected_format = Some(*format);
            break;
        }
    }

    let format = selected_format.ok_or_else(|| {
        "No supported audio format found (tried S24_3LE, S32LE, S24LE, S16LE, FloatLE)".to_string()
    })?;

    // Set channels
    hwp.set_channels(channels as u32)
        .map_err(|e| format!("Failed to set channels: {}", e))?;

    // Request the track rate. ValueOr::Nearest is still used so ALSA
    // accepts the set; we fail closed below if hardware did not match.
    hwp.set_rate(sample_rate, ValueOr::Nearest)
        .map_err(|e| format!("Failed to set sample rate: {}", e))?;

    // Set buffer size (larger buffer for high-res audio)
    let buffer_size = if sample_rate >= 192000 {
        // 500ms buffer for 192kHz+ (like MPD config)
        (sample_rate / 2) as i64
    } else if sample_rate >= 96000 {
        // 250ms buffer for 96kHz
        (sample_rate / 4) as i64
    } else {
        // 125ms buffer for lower rates
        (sample_rate / 8) as i64
    };

    hwp.set_buffer_size_near(buffer_size)
        .map_err(|e| format!("Failed to set buffer size: {}", e))?;

    // Set period size (1/10 of buffer)
    hwp.set_period_size_near(buffer_size / 10, ValueOr::Nearest)
        .map_err(|e| format!("Failed to set period size: {}", e))?;

    // Apply hardware parameters
    pcm.hw_params(&hwp)
        .map_err(|e| format!("Failed to apply hardware params: {}", e))?;

    ensure_exact_rate(&hwp, sample_rate, "exclusive PCM")?;

    log::info!(
        "[ALSA Direct] Hardware configured: {}Hz, {}ch, buffer: {} frames, format: {:?}",
        sample_rate,
        channels,
        buffer_size,
        format
    );

    Ok(format)
}
