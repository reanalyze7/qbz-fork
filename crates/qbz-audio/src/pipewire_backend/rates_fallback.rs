//! Same-family fallback-rate selection, split out of `rates.rs` to keep both
//! files under the line-count limit — pure logic, no I/O.

use super::PipeWireBackend;

impl PipeWireBackend {
    /// Find the best fallback sample rate in the same family.
    /// 44.1kHz family: 44100, 88200, 176400, 352800
    /// 48kHz family: 48000, 96000, 192000, 384000
    pub(crate) fn find_best_fallback_rate(requested: u32, supported: &[u32]) -> u32 {
        let is_441_family = requested % 44100 == 0;

        // Find highest supported rate in the same family that's <= requested
        let mut candidates: Vec<u32> = supported
            .iter()
            .filter(|&&r| {
                if is_441_family {
                    r % 44100 == 0
                } else {
                    r % 48000 == 0
                }
            })
            .filter(|&&r| r <= requested)
            .copied()
            .collect();
        candidates.sort();

        if let Some(&best) = candidates.last() {
            return best;
        }

        // No rate in the same family — use highest supported rate overall
        supported.iter().copied().max().unwrap_or(48000)
    }
}
