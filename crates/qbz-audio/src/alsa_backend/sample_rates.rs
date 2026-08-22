//! CPAL-facing sample rate probing/fallback, distinct from the `/proc`-based
//! hardware rates in `proc_rates.rs`.

/// Common audio sample rates to check for device support
pub(super) const COMMON_SAMPLE_RATES: &[u32] = &[
    44100,  // CD quality
    48000,  // DVD/DAT quality
    88200,  // 2x CD
    96000,  // DVD-Audio
    176400, // 4x CD
    192000, // High-res audio
    352800, // DSD64 equivalent
    384000, // Ultra high-res
];

/// Find the best fallback sample rate in the same family.
/// 44.1kHz family: 44100, 88200, 176400, 352800
/// 48kHz family: 48000, 96000, 192000, 384000
pub(super) fn find_best_fallback_rate(requested: u32, supported: &[u32]) -> u32 {
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

/// Extract supported sample rates from a CPAL device
pub(super) fn get_supported_sample_rates(device: &rodio::cpal::Device) -> Option<Vec<u32>> {
    use rodio::cpal::traits::DeviceTrait;

    let configs = device.supported_output_configs().ok()?;
    let configs_vec: Vec<_> = configs.collect();

    if configs_vec.is_empty() {
        return None;
    }

    let mut supported = Vec::new();

    for rate in COMMON_SAMPLE_RATES {
        let sample_rate = *rate;
        // Check if any config supports this rate
        let is_supported = configs_vec.iter().any(|config| {
            sample_rate >= config.min_sample_rate() && sample_rate <= config.max_sample_rate()
        });
        if is_supported {
            supported.push(*rate);
        }
    }

    if supported.is_empty() {
        None
    } else {
        Some(supported)
    }
}
