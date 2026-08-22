//! External streaming (Cast / DLNA) asset types.

use serde::{Deserialize, Serialize};

use super::StreamQualityInfo;

/// Measured stream parameters read from the head of an audio buffer
/// (FLAC STREAMINFO). `sample_rate` is in Hz.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioParams {
    pub sample_rate: u32,
    pub bits_per_sample: u32,
    pub channels: u16,
}

/// Parse a FLAC STREAMINFO block from the head of a stream. Returns `None`
/// for non-FLAC or short buffers — never guesses defaults (callers that need
/// a fallback keep their own). Bit math hoisted verbatim from the proven
/// QConnect remote-stream probe (`qbz::remote_stream`), shared so the cast
/// path can measure the bytes it actually serves (#638 fix 1).
pub fn probe_streaminfo(bytes: &[u8]) -> Option<AudioParams> {
    if bytes.len() >= 26 && bytes.starts_with(b"fLaC") {
        let sample_rate = ((bytes[18] as u32) << 12)
            | ((bytes[19] as u32) << 4)
            | ((bytes[20] as u32) >> 4);
        let channels = ((bytes[20] >> 1) & 0x07) + 1;
        let bit_depth = ((bytes[20] & 0x01) << 4) | ((bytes[21] >> 4) & 0x0F);
        Some(AudioParams {
            sample_rate,
            bits_per_sample: (bit_depth + 1) as u32,
            channels: channels as u16,
        })
    } else {
        None
    }
}

/// Where the bytes for an external/cast track were resolved from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetOrigin {
    Network,
    Cache,
    Offline,
}

/// A fully-materialized audio asset to hand to an external renderer
/// (Chromecast / DLNA) through the local media server. Carries the raw bytes
/// VERBATIM (no transcode), the MIME to advertise, and the quality actually
/// resolved so the UI can display it. Casting bypasses the local audio
/// backend, so this is the only place the delivered quality is known.
#[derive(Clone)]
pub struct ExternalStreamAsset {
    pub bytes: Vec<u8>,
    pub content_type: String,
    pub quality: StreamQualityInfo,
    /// Track duration in seconds, when known by the resolver.
    pub duration_secs: Option<f64>,
    pub origin: AssetOrigin,
}

impl std::fmt::Debug for ExternalStreamAsset {
    // Don't dump the whole byte vec into logs.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExternalStreamAsset")
            .field("bytes", &format_args!("{} bytes", self.bytes.len()))
            .field("content_type", &self.content_type)
            .field("quality", &self.quality)
            .field("duration_secs", &self.duration_secs)
            .field("origin", &self.origin)
            .finish()
    }
}
