//! Direct ALSA access using alsa-rs
//!
//! Provides bit-perfect playback for hw:X,Y devices that CPAL cannot open.
//! This module bypasses rodio/CPAL completely for direct hardware access.

#[cfg(target_os = "linux")]
use alsa::pcm::{Format, PCM};
#[cfg(target_os = "linux")]
use std::sync::atomic::AtomicBool;
#[cfg(target_os = "linux")]
use std::sync::{Arc, Mutex};

#[cfg(target_os = "linux")]
mod lifecycle;
#[cfg(target_os = "linux")]
mod mixer;
#[cfg(target_os = "linux")]
mod open;
#[cfg(target_os = "linux")]
mod open_dsd;
#[cfg(target_os = "linux")]
mod recovery;
#[cfg(target_os = "linux")]
mod util;
#[cfg(target_os = "linux")]
mod write_dop;
#[cfg(target_os = "linux")]
mod write_pcm;

#[cfg(not(target_os = "linux"))]
mod stub;

/// Direct ALSA PCM stream for hw: devices
///
/// Field order is significant: Rust drops struct fields top-to-bottom, so the
/// `PCM` is dropped first (releasing the kernel-level exclusive grip on the
/// `hw:` device) BEFORE `_reservation` drops (releasing the
/// `org.freedesktop.ReserveDevice1.Audio<N>` bus name back to PipeWire).
///
/// Reversing this order would tell PipeWire "go ahead, take the device" while
/// the kernel still has the FD open — guaranteed `EBUSY` ping-pong on the next
/// stream open. `_reservation` is intentionally the last field for that
/// reason; do not rearrange.
#[cfg(target_os = "linux")]
pub struct AlsaDirectStream {
    pcm: Arc<Mutex<PCM>>,
    #[allow(dead_code)]
    is_playing: Arc<AtomicBool>,
    sample_rate: u32,
    channels: u16,
    format: Format,
    device_id: String,
    /// D-Bus device reservation held for the entire stream lifetime
    /// (Lifetime A per the design spec). Acquired before `PCM::new()` in
    /// `Self::new()`; released on `Drop` *after* the PCM closes (see field-order
    /// note on the struct above).
    _reservation: crate::DeviceReservation,
}

#[cfg(not(target_os = "linux"))]
pub struct AlsaDirectStream {
    sample_rate: u32,
    channels: u16,
    device_id: String,
}

/// Defensive settle delay between reservation acquisition and PCM open.
///
/// Only applied when the reservation actually transitioned ownership (i.e.
/// `DeviceReservation::is_active()` is `true`). Sized conservatively; do not
/// reduce without revisiting the Lifetime-A safety contract in
/// `qbz-nix-docs/specs/2026-05-07-alsa-exclusive-hardening-design.md`.
#[cfg(target_os = "linux")]
const PIPEWIRE_VACATE_MARGIN: std::time::Duration = std::time::Duration::from_millis(50);
