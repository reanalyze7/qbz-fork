//! Dynamic-suggest seed types and audio quality tiers.

use serde::{Deserialize, Serialize};

/// A seed track resolved for the `/dynamic/suggest` `track_to_analysed`
/// payload (DailyQ / WeeklyQ). Field names match the Qobuz wire shape
/// exactly; `0` marks an unknown id (mirrors Tauri's `?? 0`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackToAnalyse {
    pub track_id: u64,
    pub artist_id: u64,
    pub genre_id: u64,
    pub label_id: u64,
}

/// Audio quality format IDs (matches Qobuz API format IDs)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u32)]
pub enum Quality {
    Mp3 = 5,
    Lossless = 6,    // 16-bit/44.1kHz (CD Quality)
    HiRes = 7,       // 24-bit/≤96kHz
    UltraHiRes = 27, // 24-bit/>96kHz
}

impl Quality {
    pub fn from_id(id: u32) -> Option<Self> {
        match id {
            5 => Some(Quality::Mp3),
            6 => Some(Quality::Lossless),
            7 => Some(Quality::HiRes),
            27 => Some(Quality::UltraHiRes),
            _ => None,
        }
    }

    pub fn id(&self) -> u32 {
        *self as u32
    }

    pub fn label(&self) -> &'static str {
        match self {
            Quality::Mp3 => "MP3 320kbps",
            Quality::Lossless => "FLAC 16-bit/44.1kHz",
            Quality::HiRes => "FLAC 24-bit/≤96kHz",
            Quality::UltraHiRes => "FLAC 24-bit/>96kHz",
        }
    }

    /// Quality levels in descending order for fallback
    pub fn fallback_order() -> &'static [Quality] {
        &[
            Quality::UltraHiRes,
            Quality::HiRes,
            Quality::Lossless,
            Quality::Mp3,
        ]
    }

    /// Returns the next lower quality level, or None if already at the lowest (Mp3).
    /// Used for CDN fallback when a quality level consistently fails.
    pub fn lower(&self) -> Option<Quality> {
        match self {
            Quality::UltraHiRes => Some(Quality::HiRes),
            Quality::HiRes => Some(Quality::Lossless),
            Quality::Lossless => Some(Quality::Mp3),
            Quality::Mp3 => None,
        }
    }

    /// The lower of two tiers. Implementable as plain `min` because the
    /// derived `Ord` on `Quality` is tier-correct: the Qobuz format-id
    /// discriminants (5 Mp3 < 6 Lossless < 7 HiRes < 27 UltraHiRes) ascend
    /// with tier. Used to clamp a requested tier against a cap (#638).
    pub fn min_tier(a: Quality, b: Quality) -> Quality {
        a.min(b)
    }
}

impl Default for Quality {
    fn default() -> Self {
        Quality::Lossless
    }
}

/// Why a delivered stream is (or may be) below the track's catalog maximum.
/// Shared by the local badge, the local device cap, and the cast surfaces
/// (#638 fixes 1-4). Mirrored to Slint as a plain `int` property carrying the
/// same discriminant — no string enum crosses the FFI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QualityLimit {
    /// No constraint identified (or no downgrade).
    #[default]
    None = 0,
    /// The user's streaming-quality preference capped the request.
    Preference = 1,
    /// The local output device's cap lowered the request (fix 3).
    /// NEVER applicable while casting — the local DAC is not in a cast's
    /// signal path (precedence rule, owner decision 2026-07-20).
    LocalDeviceCap = 2,
    /// The manual per-renderer cap lowered the request (fix 4). Cast only.
    RendererCap = 3,
    /// Qobuz did not offer a higher tier for this track.
    CatalogAvailability = 4,
}
