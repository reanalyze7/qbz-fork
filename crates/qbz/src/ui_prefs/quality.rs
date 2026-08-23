//! Streaming-quality tiers and key<->index/format mapping.

use qbz_models::Quality;

/// Streaming-quality tiers, mirroring the Tauri app's dropdown. The
/// `format_id` is the Qobuz format identifier the request layer expects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamingQuality {
    /// Stable key persisted to JSON.
    pub key: &'static str,
    /// Human-facing label for the dropdown.
    pub label: &'static str,
}

/// The four streaming-quality options, in on-screen order.
pub const STREAMING_QUALITIES: &[StreamingQuality] = &[
    StreamingQuality { key: "mp3", label: "MP3" },
    StreamingQuality { key: "cd", label: "CD Quality" },
    StreamingQuality { key: "hires", label: "Hi-Res" },
    StreamingQuality { key: "hires_plus", label: "Hi-Res+" },
];

/// Default streaming-quality key (`Hi-Res+`).
pub const DEFAULT_STREAMING_QUALITY: &str = "hires_plus";

/// Map a persisted streaming-quality key to the Qobuz format id the
/// request layer expects (`Quality`). Unknown/unset keys fall back to the
/// default tier (`Hi-Res+` = `Quality::UltraHiRes`), mirroring
/// `streaming_quality_index`.
pub fn streaming_quality_for_key(key: &str) -> Quality {
    match key {
        "mp3" => Quality::Mp3,
        "cd" => Quality::Lossless,
        "hires" => Quality::HiRes,
        _ => Quality::UltraHiRes, // "hires_plus" + unknown keys
    }
}

/// Index of `key` in `STREAMING_QUALITIES`, falling back to the default
/// (`Hi-Res+`) when the stored key is unknown.
pub fn streaming_quality_index(key: &str) -> usize {
    STREAMING_QUALITIES
        .iter()
        .position(|q| q.key == key)
        .unwrap_or_else(|| {
            STREAMING_QUALITIES
                .iter()
                .position(|q| q.key == DEFAULT_STREAMING_QUALITY)
                .unwrap_or(0)
        })
}

pub(super) fn default_streaming_quality() -> String {
    DEFAULT_STREAMING_QUALITY.to_string()
}
