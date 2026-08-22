//! ALSA Direct stream error classification and bit-perfect mode tracking.
//! Pure classification logic, no I/O.

use serde::{Deserialize, Serialize};

/// ALSA Direct stream error classification
/// Used to determine if fallback to plughw is appropriate
#[derive(Debug, Clone)]
pub enum AlsaDirectError {
    /// PCM format not supported by hardware (can fallback to plughw)
    UnsupportedFormat(String),
    /// Device is busy/in use by another application
    DeviceBusy(String),
    /// Permission denied to access device
    PermissionDenied(String),
    /// Invalid parameters (channels, sample rate)
    InvalidParams(String),
    /// Device not found
    DeviceNotFound(String),
    /// Generic/unknown error
    Other(String),
}

impl std::fmt::Display for AlsaDirectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlsaDirectError::UnsupportedFormat(msg) => write!(f, "Unsupported format: {}", msg),
            AlsaDirectError::DeviceBusy(msg) => write!(f, "Device busy: {}", msg),
            AlsaDirectError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            AlsaDirectError::InvalidParams(msg) => write!(f, "Invalid parameters: {}", msg),
            AlsaDirectError::DeviceNotFound(msg) => write!(f, "Device not found: {}", msg),
            AlsaDirectError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl AlsaDirectError {
    /// Check if this error allows fallback to plughw
    pub fn allows_plughw_fallback(&self) -> bool {
        matches!(self, AlsaDirectError::UnsupportedFormat(_))
    }

    /// Create from raw ALSA error message
    pub fn from_alsa_error(msg: &str) -> Self {
        let msg_lower = msg.to_lowercase();

        if msg_lower.contains("no supported audio format")
            || msg_lower.contains("format")
            || msg_lower.contains("s24_3le")
            || msg_lower.contains("s24le")
            || msg_lower.contains("sample format")
        {
            AlsaDirectError::UnsupportedFormat(msg.to_string())
        } else if msg_lower.contains("busy")
            || msg_lower.contains("resource temporarily unavailable")
            || msg_lower.contains("device or resource busy")
        {
            AlsaDirectError::DeviceBusy(msg.to_string())
        } else if msg_lower.contains("permission")
            || msg_lower.contains("access denied")
            || msg_lower.contains("operation not permitted")
        {
            AlsaDirectError::PermissionDenied(msg.to_string())
        } else if msg_lower.contains("not found")
            || msg_lower.contains("no such")
            || msg_lower.contains("doesn't exist")
        {
            AlsaDirectError::DeviceNotFound(msg.to_string())
        } else if msg_lower.contains("invalid")
            || msg_lower.contains("channels")
            || msg_lower.contains("rate")
        {
            AlsaDirectError::InvalidParams(msg.to_string())
        } else {
            AlsaDirectError::Other(msg.to_string())
        }
    }
}

/// Runtime mode for bit-perfect status tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BitPerfectMode {
    /// Direct hardware access (hw:), guaranteed bit-perfect
    DirectHardware,
    /// Plugin hardware fallback (plughw:), bit-perfect with format conversion only
    PluginFallback,
    /// Not using bit-perfect path (pcm, pipewire, pulse)
    Disabled,
}
