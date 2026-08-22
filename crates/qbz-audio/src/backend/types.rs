//! Cross-backend enum types: `AudioBackendType`, `AlsaPlugin`. Pure data, no
//! logic. `AudioDevice`/`BackendConfig`/`BackendResult` live in
//! `device_config.rs` to keep this file under the line-count limit.

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

