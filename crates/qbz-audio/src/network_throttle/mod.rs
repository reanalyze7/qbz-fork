//! Adaptive prefetch throttling based on observed network conditions.
//!
//! Default QBZ prefetches up to 5 tracks concurrently (≈30 simultaneous
//! HTTP/2 streams once the CMAF init + segment fan-out is counted). That
//! aggressive concurrency is exactly what makes Hi-Res playback smooth on
//! a fast connection, and we don't want to dial it down for everyone just
//! because some users have a slow link.
//!
//! Instead, this module watches two real-time signals — measured per-segment
//! bandwidth and audio underruns — and computes a *dynamic* cap that the
//! prefetch dispatcher consults. When conditions are good the cap is the
//! full memory-profile default (5 on Normal-class hosts, 1 on LowMemory).
//! When the live stream starts losing the race against the consumer, the
//! cap collapses to 0 and the prefetch fan-out gets out of the way.
//!
//! Recovery follows TCP-style slow-start logic: after `PANIC_WINDOW_SECS`
//! of no fresh underruns the cap walks back up one level at a time,
//! re-validated by each new bandwidth sample.

mod state;
#[cfg(test)]
mod tests;

pub use state::{state, ThrottleState};

/// EMA smoothing factor for observed bandwidth. Higher = more reactive to
/// the latest sample, lower = smoother but laggier. 0.4 is a compromise:
/// a single bad sample dents the estimate but doesn't dominate it.
const BANDWIDTH_EMA_ALPHA: f64 = 0.4;

/// How long we stay in panic mode after an audio underrun. The user just
/// experienced a glitch; we want the stream to recover with the entire
/// pipe to itself for a meaningful window before letting prefetch back in.
const PANIC_WINDOW_SECS: u64 = 30;

/// Bandwidth-to-playback ratio thresholds for the throttle levels.
///
/// - At or below `SURVIVING_RATIO`: no prefetch at all. The live stream
///   barely has bandwidth to keep itself fed.
/// - At or below `CAUTIOUS_RATIO`: only one prefetch track in flight.
/// - At or below `RELAXED_RATIO`: two prefetch tracks.
/// - Above `RELAXED_RATIO`: full memory-profile default.
const SURVIVING_RATIO: f64 = 1.5;
const CAUTIOUS_RATIO: f64 = 2.5;
const RELAXED_RATIO: f64 = 4.0;

/// Approximate sustained bandwidth required for live playback by quality
/// tier, in MB/s. These numbers are for CMAF/FLAC compressed streams;
/// they're inputs to the ratio comparison above, not hard limits.
pub fn playback_mbps_for_quality(quality_tag: PlaybackQualityTag) -> f64 {
    match quality_tag {
        PlaybackQualityTag::UltraHiRes => 2.5, // 24-bit / 192 kHz FLAC
        PlaybackQualityTag::HiRes => 1.4,      // 24-bit / 96 kHz FLAC
        PlaybackQualityTag::Lossless => 0.5,   // 16-bit / 44.1 kHz FLAC
        PlaybackQualityTag::Lossy => 0.04,     // 320 kbps MP3
    }
}

/// Small enum that mirrors `qbz-models::Quality` without taking a runtime
/// dependency on it. The callsite translates its own quality enum into one
/// of these four buckets before consulting the throttle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackQualityTag {
    UltraHiRes,
    HiRes,
    Lossless,
    Lossy,
}
