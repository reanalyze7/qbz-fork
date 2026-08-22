//! Cross-backend enums/config types: `AudioBackendType`, `AlsaPlugin`,
//! `AudioDevice`, `BackendConfig`, `BackendResult`. Pure data, no logic.

use serde::{Deserialize, Serialize};

/// Supported audio backends
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioBackendType {
    /// PipeWire backend (modern, recommended)
    /// - Supports device selection without changing system default
    /// - Uses PULSE_SINK environment variable
    /// - Compatible with PulseAudio apps
    PipeWire,

    /// ALSA backend (direct hardware access)
    /// - True exclusive mode (blocks device for other apps)
    /// - Bit-perfect guaranteed
    /// - Lowest latency
    /// - Requires manual device selection (hw:X,Y)
    Alsa,

    /// PulseAudio backend (legacy compatibility)
    /// - Similar to PipeWire but older
    /// - Fallback for systems without PipeWire
    Pulse,

    /// JACK backend (#263 Tier 3 — pro-audio routing). Linux-only in practice.
    /// - QBZ appears as a first-class JACK client with stable ports
    ///   (`qbz:out_FL` / `qbz:out_FR`), patchable in qjackctl/qpwgraph/Reaper
    /// - Routing survives track changes (the client + ports live once)
    /// - NOT bit-perfect: the JACK graph runs at ONE fixed rate (audio is
    ///   resampled) — an opt-in routing-freedom mode; never touches the
    ///   bit-perfect ALSA-exclusive / DAC-passthrough paths.
    Jack,

    /// System default backend (non-Linux platforms)
    /// - Uses CPAL default host (CoreAudio on macOS, WASAPI on Windows)
    /// - Automatic device selection via OS audio system
    SystemDefault,
}

impl Default for AudioBackendType {
    fn default() -> Self {
        // "System" everywhere: the OOTB default plays through the OS default
        // output, shared with other apps (no bit-perfect, no `pactl`). Audiophile
        // users opt into PipeWire / ALSA explicitly. This was PipeWire on Linux,
        // which hard-required `pactl` and froze OOTB playback without it (#470).
        AudioBackendType::SystemDefault
    }
}

/// ALSA plugin type (only relevant for ALSA backend)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlsaPlugin {
    /// Direct hardware access (hw)
    /// - Bit-perfect, exclusive
    /// - No automatic format conversion
    /// - Blocks device for other apps
    Hw,

    /// Plug hardware access (plughw)
    /// - Automatic format conversion
    /// - Resampling if needed
    /// - Still relatively direct
    PlugHw,

    /// PCM device (default)
    /// - Generic ALSA device
    /// - Most compatible
    Pcm,
}

impl Default for AlsaPlugin {
    fn default() -> Self {
        // Hw is the audiophile choice
        AlsaPlugin::Hw
    }
}

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
