//! Effective-sample-rate resolution for `AlsaBackend::create_output_stream`,
//! split out of that function purely to stay under the per-file line
//! budget — behavior is unchanged.

use super::sample_rates::{find_best_fallback_rate, get_supported_sample_rates};
use crate::backend::BackendConfig;
use rodio::cpal::traits::DeviceTrait;

/// Determine the sample rate to actually open the device at: `config.sample_rate`
/// if the device reports support for it, otherwise the best same-family
/// fallback (rodio will resample).
pub(super) fn determine_effective_rate(
    device: &rodio::cpal::Device,
    config: &BackendConfig,
) -> Result<u32, String> {
    // Check if device supports this configuration
    let supported_configs = device
        .supported_output_configs()
        .map_err(|e| format!("Failed to get supported configs: {}", e))?;

    let mut found_matching = false;
    for range in supported_configs {
        if range.channels() == config.channels
            && config.sample_rate >= range.min_sample_rate()
            && config.sample_rate <= range.max_sample_rate()
        {
            found_matching = true;
            log::info!(
                "[ALSA Backend] Device supports {}Hz (range: {}-{}Hz)",
                config.sample_rate,
                range.min_sample_rate(),
                range.max_sample_rate()
            );
            break;
        }
    }

    // If device doesn't support the requested rate, find best fallback
    let effective_rate = if !found_matching {
        if let Some(rates) = get_supported_sample_rates(device) {
            let fallback = find_best_fallback_rate(config.sample_rate, &rates);
            log::warn!(
                "[ALSA Backend] Device doesn't support {}Hz. Supported: {:?}. Falling back to {}Hz (rodio will resample)",
                config.sample_rate, rates, fallback
            );
            fallback
        } else {
            log::warn!(
                "[ALSA Backend] Could not determine supported rates, attempting {}Hz anyway",
                config.sample_rate
            );
            config.sample_rate
        }
    } else {
        config.sample_rate
    };

    Ok(effective_rate)
}
