//! `AlsaDirectStream::new` — standard PCM open with format-priority
//! negotiation: reservation-acquire, PCM open, hwparams negotiation
//! (delegated to [`negotiate`]), and `prepare()`.

mod negotiate;

use super::{AlsaDirectStream, PIPEWIRE_VACATE_MARGIN};
use alsa::pcm::PCM;
use alsa::Direction;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

impl AlsaDirectStream {
    /// Create new ALSA direct stream
    pub fn new(device_id: &str, sample_rate: u32, channels: u16) -> Result<Self, String> {
        log::info!(
            "[ALSA Direct] Opening device: {} ({}Hz, {}ch)",
            device_id,
            sample_rate,
            channels
        );

        // Acquire D-Bus device reservation BEFORE opening the PCM. This signals
        // PipeWire/WirePlumber to release the device first if it currently
        // holds it. Held for the entire `AlsaDirectStream` lifetime
        // (Lifetime A per the design spec) and released on `Drop` after the
        // PCM closes — see the field-order comment on the struct.
        //
        // This is the canonical Lifetime-A consumer the `acquire` doc-comment's
        // tight-coupling rule allows: a `DeviceReservation` is created
        // immediately before a real `PCM::new()` and held for as long as that
        // PCM is open.
        // TODO(Task 5): replace second arg with user-facing DAC name from settings.
        let reservation = crate::DeviceReservation::acquire(device_id, device_id)
            .map_err(|e| format!("Cannot acquire exclusive device '{}': {}", device_id, e))?;

        // Defensive margin only matters when the reservation actually displaced
        // a holder (or could have). On the degraded D-Bus path the bus name is
        // not held at all, so PipeWire's view of the device hasn't changed and
        // no settle delay is needed. PIPEWIRE_VACATE_MARGIN is conservative;
        // PipeWire-side release latency is typically much shorter, but this
        // margin is part of the design spec's Lifetime-A safety contract — do
        // not reduce without revisiting the spec.
        if reservation.is_active() {
            std::thread::sleep(PIPEWIRE_VACATE_MARGIN);
        }

        // Open PCM device
        let pcm = PCM::new(device_id, Direction::Playback, false)
            .map_err(|e| format!("Failed to open ALSA device '{}': {}", device_id, e))?;

        // Set hardware parameters and auto-detect best format
        let selected_format = negotiate::negotiate(&pcm, sample_rate, channels)?;

        // Prepare device for playback
        pcm.prepare()
            .map_err(|e| format!("Failed to prepare PCM: {}", e))?;

        Ok(Self {
            pcm: Arc::new(Mutex::new(pcm)),
            is_playing: Arc::new(AtomicBool::new(false)),
            sample_rate,
            channels,
            format: selected_format,
            device_id: device_id.to_string(),
            // Last field: drops after `pcm` so the kernel-level exclusive
            // grip is released before the D-Bus bus name is freed.
            _reservation: reservation,
        })
    }
}
