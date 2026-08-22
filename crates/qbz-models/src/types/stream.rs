//! Stream URL + resolved quality info.

use serde::{Deserialize, Serialize};

use super::Quality;

/// Stream URL response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamUrl {
    pub url: String,
    pub format_id: u32,
    pub mime_type: String,
    pub sampling_rate: f64,
    pub bit_depth: Option<u32>,
    pub track_id: u64,
    pub restrictions: Vec<StreamRestriction>,
}

impl StreamUrl {
    /// Check if the stream has restrictions that prevent playback
    pub fn has_restrictions(&self) -> bool {
        self.restrictions.iter().any(|r| {
            r.code == "FormatRestrictedByFormatAvailability"
                || r.code == "SampleRestrictedByRightHolders"
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamRestriction {
    pub code: String,
}

/// Resolved audio quality actually delivered for an external stream, in the
/// kHz convention used across the catalog and [`StreamUrl`]. Surfaced so the
/// UI can show the REAL quality of a cast stream, which can fall back below
/// the requested tier (HiRes -> Lossless -> Mp3) without the user knowing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamQualityInfo {
    /// Qobuz format id: 5=MP3, 6=Lossless, 7=HiRes, 27=UltraHiRes.
    pub format_id: u32,
    /// Sampling rate in kHz (e.g. 96.0, 192.0), when known.
    pub sampling_rate_khz: Option<f64>,
    /// Bit depth (16 / 24), when known.
    pub bit_depth: Option<u32>,
}

impl StreamQualityInfo {
    /// Build from a raw sampling-rate value whose unit may be kHz or Hz
    /// depending on the Qobuz endpoint (`get_stream_url` reports kHz as f64,
    /// `file/url` reports an integer that has been observed as kHz). Normalize
    /// to kHz robustly: any real audio rate is < 1000 kHz and >= 8000 Hz, so a
    /// value >= 1000 is Hz and gets divided. Zero/negative -> unknown.
    pub fn from_raw(format_id: u32, raw_rate: Option<f64>, bit_depth: Option<u32>) -> Self {
        let sampling_rate_khz = raw_rate.and_then(|r| {
            if r <= 0.0 {
                None
            } else if r >= 1000.0 {
                Some(r / 1000.0)
            } else {
                Some(r)
            }
        });
        Self {
            format_id,
            sampling_rate_khz,
            bit_depth,
        }
    }

    /// The `Quality` tier this format id maps to, if recognized.
    pub fn quality(&self) -> Option<Quality> {
        Quality::from_id(self.format_id)
    }

    /// Coarse tier label like "FLAC 24-bit/>96kHz" (from the format id).
    pub fn tier_label(&self) -> &'static str {
        self.quality().map(|q| q.label()).unwrap_or("Unknown")
    }
}
