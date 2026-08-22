//! `AudioDevice`, `BackendConfig`, `BackendResult` — split out from
//! `types.rs` to keep that file under the line-count limit.

use super::types::{AlsaPlugin, AudioBackendType};
use serde::{Deserialize, Serialize};

/// Audio device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    /// Internal device identifier (e.g., "hw:4,0" for ALSA, sink name for PipeWire)
    pub id: String,

    /// User-friendly display name
    pub name: String,

    /// Detailed description (optional)
    pub description: Option<String>,

    /// Whether this is the system default device
    pub is_default: bool,

    /// Maximum supported sample rate (if known)
    pub max_sample_rate: Option<u32>,

    /// Supported sample rates (common audio rates that the device supports)
    /// Contains values like 44100, 48000, 88200, 96000, 176400, 192000, etc.
    pub supported_sample_rates: Option<Vec<u32>>,

    /// Device bus type (for PipeWire): "usb", "pci", "bluetooth", or None
    pub device_bus: Option<String>,

    /// Whether this is a hardware device (has HARDWARE flag in PipeWire)
    pub is_hardware: bool,
}

/// Audio backend configuration
#[derive(Debug, Clone)]
pub struct BackendConfig {
    /// Backend type
    pub backend_type: AudioBackendType,

    /// Device ID (backend-specific)
    pub device_id: Option<String>,

    /// ALSA plugin (only used if backend_type == Alsa)
    pub alsa_plugin: Option<AlsaPlugin>,

    /// Sample rate (for stream creation)
    pub sample_rate: u32,

    /// Channels
    pub channels: u16,

    /// Exclusive mode flag
    pub exclusive_mode: bool,

    /// When true, force PipeWire clock.force-quantum for bit-perfect playback
    pub pw_force_bitperfect: bool,

    /// When true, skip `pactl set-default-sink` on stream creation.
    /// Preserves external routing (JACK, qjackctl, Reaper).
    pub skip_sink_switch: bool,
}

/// Result type for backend operations
pub type BackendResult<T> = Result<T, String>;
