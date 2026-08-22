//! Supported audio format enum

use serde::{Deserialize, Serialize};

/// Supported audio formats
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AudioFormat {
    Flac,
    Alac,
    Wav,
    Aiff,
    Ape,
    Mp3,
    Dsd,
    Unknown,
}

impl Default for AudioFormat {
    fn default() -> Self {
        Self::Unknown
    }
}

impl std::fmt::Display for AudioFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioFormat::Flac => write!(f, "FLAC"),
            AudioFormat::Alac => write!(f, "ALAC"),
            AudioFormat::Wav => write!(f, "WAV"),
            AudioFormat::Aiff => write!(f, "AIFF"),
            AudioFormat::Ape => write!(f, "APE"),
            AudioFormat::Mp3 => write!(f, "MP3"),
            AudioFormat::Dsd => write!(f, "DSD"),
            AudioFormat::Unknown => write!(f, "Unknown"),
        }
    }
}
