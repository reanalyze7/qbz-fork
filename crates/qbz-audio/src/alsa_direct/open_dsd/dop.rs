//! `AlsaDirectStream::new_dop`.

use crate::alsa_direct::recovery::ensure_exact_rate;
use crate::alsa_direct::{AlsaDirectStream, PIPEWIRE_VACATE_MARGIN};
use alsa::pcm::{Access, Format, HwParams, PCM};
use alsa::{Direction, ValueOr};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

impl AlsaDirectStream {
    /// Create an ALSA direct stream for DoP (DSD over PCM) delivery.
    ///
    /// ADDITIVE to the protected PCM paths (DSD plan Phase 2, owner-approved
    /// 2026-07-03): S32_LE ONLY — DoP words are pre-packed 24-bit frames
    /// left-justified in S32 and must reach the device bit-exactly, so no
    /// format fallback, no plughw, no float. If the device has no S32_LE at
    /// the carrier rate the caller falls back to DSD→PCM conversion.
    /// Mirrors `new()` for reservation / buffer sizing / field order.
    pub fn new_dop(device_id: &str, carrier_rate: u32, channels: u16) -> Result<Self, String> {
        log::info!(
            "[ALSA Direct] Opening device for DoP: {} ({}Hz carrier, {}ch, S32_LE)",
            device_id,
            carrier_rate,
            channels
        );

        let reservation = crate::DeviceReservation::acquire(device_id, device_id)
            .map_err(|e| format!("Cannot acquire exclusive device '{}': {}", device_id, e))?;
        if reservation.is_active() {
            std::thread::sleep(PIPEWIRE_VACATE_MARGIN);
        }

        let pcm = PCM::new(device_id, Direction::Playback, false)
            .map_err(|e| format!("Failed to open ALSA device '{}': {}", device_id, e))?;

        {
            let hwp =
                HwParams::any(&pcm).map_err(|e| format!("Failed to get hardware params: {}", e))?;
            hwp.set_access(Access::RWInterleaved)
                .map_err(|e| format!("Failed to set access: {}", e))?;
            hwp.set_format(Format::S32LE)
                .map_err(|e| format!("Device has no S32_LE (required for DoP): {}", e))?;
            hwp.set_channels(channels as u32)
                .map_err(|e| format!("Failed to set channels: {}", e))?;
            hwp.set_rate(carrier_rate, ValueOr::Nearest)
                .map_err(|e| format!("Failed to set DoP carrier rate {}: {}", carrier_rate, e))?;
            let buffer_size = if carrier_rate >= 192000 {
                (carrier_rate / 2) as i64
            } else if carrier_rate >= 96000 {
                (carrier_rate / 4) as i64
            } else {
                (carrier_rate / 8) as i64
            };
            hwp.set_buffer_size_near(buffer_size)
                .map_err(|e| format!("Failed to set buffer size: {}", e))?;
            hwp.set_period_size_near(buffer_size / 10, ValueOr::Nearest)
                .map_err(|e| format!("Failed to set period size: {}", e))?;
            pcm.hw_params(&hwp)
                .map_err(|e| format!("Failed to apply hardware params: {}", e))?;
            ensure_exact_rate(&hwp, carrier_rate, "DoP carrier")?;
            log::info!(
                "[ALSA Direct] DoP hardware configured: {}Hz, {}ch, S32_LE, buffer {} frames",
                carrier_rate,
                channels,
                buffer_size
            );
        }

        pcm.prepare()
            .map_err(|e| format!("Failed to prepare PCM: {}", e))?;

        Ok(Self {
            pcm: Arc::new(Mutex::new(pcm)),
            is_playing: Arc::new(AtomicBool::new(false)),
            sample_rate: carrier_rate,
            channels,
            format: Format::S32LE,
            device_id: device_id.to_string(),
            // Last field: drops after `pcm` (see field-order note on the struct).
            _reservation: reservation,
        })
    }
}
