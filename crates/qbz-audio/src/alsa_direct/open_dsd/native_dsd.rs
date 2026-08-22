//! `AlsaDirectStream::new_native_dsd`.

use crate::alsa_direct::recovery::ensure_exact_rate;
use crate::alsa_direct::{AlsaDirectStream, PIPEWIRE_VACATE_MARGIN};
use alsa::pcm::{Access, Format, HwParams, PCM};
use alsa::{Direction, ValueOr};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

impl AlsaDirectStream {
    /// Create an ALSA direct stream for NATIVE DSD (DSD plan Phase 3).
    ///
    /// ADDITIVE like `new_dop`. Tries `DSD_U32_BE` first (what the kernel's
    /// generic USB DSD quirk grants), then `DSD_U32_LE`. Frame rate =
    /// dsd_rate / 32. Returns the stream plus `little_endian` so the packer
    /// lays the 4 DSD bytes out correctly. Fails cleanly when the kernel
    /// hasn't granted the device a DSD format (no quirk) — the caller falls
    /// back to DoP/conversion.
    pub fn new_native_dsd(
        device_id: &str,
        dsd_rate: u32,
        channels: u16,
    ) -> Result<(Self, bool), String> {
        let rate = dsd_rate / 32;
        log::info!(
            "[ALSA Direct] Opening device for native DSD: {} ({} DSD bits/s → {} Hz U32, {}ch)",
            device_id,
            dsd_rate,
            rate,
            channels
        );

        let reservation = crate::DeviceReservation::acquire(device_id, device_id)
            .map_err(|e| format!("Cannot acquire exclusive device '{}': {}", device_id, e))?;
        if reservation.is_active() {
            std::thread::sleep(PIPEWIRE_VACATE_MARGIN);
        }

        let pcm = PCM::new(device_id, Direction::Playback, false)
            .map_err(|e| format!("Failed to open ALSA device '{}': {}", device_id, e))?;

        let selected = {
            let hwp =
                HwParams::any(&pcm).map_err(|e| format!("Failed to get hardware params: {}", e))?;
            hwp.set_access(Access::RWInterleaved)
                .map_err(|e| format!("Failed to set access: {}", e))?;
            let mut selected = None;
            for (format, le) in [(Format::DSDU32BE, false), (Format::DSDU32LE, true)] {
                if hwp.set_format(format).is_ok() {
                    selected = Some((format, le));
                    break;
                }
            }
            let Some((format, le)) = selected else {
                return Err(
                    "Device has no native DSD format (kernel quirk missing?)".to_string()
                );
            };
            hwp.set_channels(channels as u32)
                .map_err(|e| format!("Failed to set channels: {}", e))?;
            hwp.set_rate(rate, ValueOr::Nearest)
                .map_err(|e| format!("Failed to set native DSD rate {}: {}", rate, e))?;
            let buffer_size = (rate / 4) as i64; // 250 ms
            hwp.set_buffer_size_near(buffer_size)
                .map_err(|e| format!("Failed to set buffer size: {}", e))?;
            hwp.set_period_size_near(buffer_size / 10, ValueOr::Nearest)
                .map_err(|e| format!("Failed to set period size: {}", e))?;
            pcm.hw_params(&hwp)
                .map_err(|e| format!("Failed to apply hardware params: {}", e))?;
            ensure_exact_rate(&hwp, rate, "native DSD")?;
            log::info!(
                "[ALSA Direct] Native DSD configured: {:?} @ {} Hz, {}ch",
                format,
                rate,
                channels
            );
            (format, le)
        };

        pcm.prepare()
            .map_err(|e| format!("Failed to prepare PCM: {}", e))?;

        Ok((
            Self {
                pcm: Arc::new(Mutex::new(pcm)),
                is_playing: Arc::new(AtomicBool::new(false)),
                sample_rate: rate,
                channels,
                format: selected.0,
                device_id: device_id.to_string(),
                // Last field: drops after `pcm` (see field-order note).
                _reservation: reservation,
            },
            selected.1,
        ))
    }
}
