//! Output sink enumeration (frontend-shaped diagnostic).
//!
//! Provides a small, frontend-facing struct (`OutputSinkInfo`) listing the
//! available CPAL output devices. This is the same data the legacy
//! `get_pipewire_sinks` command exposed — the simpler shape (`name`,
//! `description`, `volume`, `is_default`) used by the audio settings UI
//! and the AudioOutputBadges component.
//!
//! NOTE: This is the same CPAL host the Player itself opens, so the
//! `name` returned here is guaranteed to be a valid identifier the
//! audio backend can re-open later. It is intentionally NOT the richer
//! `AudioDevice` struct from `backend::AudioBackend::enumerate_devices`,
//! which carries sample-rate probing data the Settings UI does not need.

#[cfg(target_os = "linux")]
mod linux;
#[cfg(not(target_os = "linux"))]
mod other;

use serde::Serialize;

#[cfg(target_os = "linux")]
pub use linux::list_output_sinks;
#[cfg(not(target_os = "linux"))]
pub use other::list_output_sinks;

/// Frontend-shaped info for a single audio output device.
///
/// Mirrors the legacy `PipewireSink` struct so the existing TypeScript
/// `PipewireSink` interface can consume V2 output unchanged.
#[derive(Debug, Clone, Serialize)]
pub struct OutputSinkInfo {
    /// Internal name (e.g. CPAL device name; on Linux this is the
    /// PipeWire/PulseAudio sink name like `alsa_output.usb-XXX`).
    pub name: String,
    /// User-friendly description. On PipeWire the CPAL name is already
    /// user-readable; on macOS/Windows the name itself is descriptive.
    pub description: String,
    /// Current volume percentage (0–100). CPAL does not expose this so
    /// it is always `None` here; preserved for API compatibility.
    pub volume: Option<u32>,
    /// Whether this is the default sink.
    pub is_default: bool,
}

/// Resolve the CPAL `description().name()` for a device, returning `None`
/// if the description cannot be queried.
fn cpal_device_name(device: &rodio::cpal::Device) -> Option<String> {
    use rodio::cpal::traits::DeviceTrait;
    device
        .description()
        .ok()
        .map(|description| description.name().to_string())
}
