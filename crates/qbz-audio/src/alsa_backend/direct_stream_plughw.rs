//! `plughw:` fallback attempt (with busy-retry backoff) for
//! `AlsaBackend::try_create_direct_stream`, split out of that function
//! purely to stay under the per-file line budget — behavior is unchanged.

use super::pipewire_suspend::suspend_default_sink_for_exclusive;
use crate::backend::{AlsaDirectError, BackendConfig, BitPerfectMode};
use crate::AlsaDirectStream;

pub(super) fn try_plughw_attempt(
    plughw_device: &str,
    config: &BackendConfig,
) -> Option<Result<(AlsaDirectStream, BitPerfectMode), String>> {
    log::info!(
        "[ALSA Backend] Attempting plughw stream: {} ({}Hz, {}ch)",
        plughw_device,
        config.sample_rate,
        config.channels
    );

    match AlsaDirectStream::new(plughw_device, config.sample_rate, config.channels) {
        Ok(stream) => {
            log::info!(
                "[ALSA Backend] ✓ plughw stream created (bit-perfect with format conversion)"
            );
            Some(Ok((stream, BitPerfectMode::PluginFallback)))
        }
        Err(e) => {
            let error = AlsaDirectError::from_alsa_error(&e);

            if matches!(error, AlsaDirectError::DeviceBusy(_)) {
                log::info!("[ALSA Backend] plughw device busy — retrying with backoff");
                suspend_default_sink_for_exclusive();

                let retry_delays_ms = [50, 100, 200, 400, 800];
                for (i, delay_ms) in retry_delays_ms.iter().enumerate() {
                    std::thread::sleep(std::time::Duration::from_millis(*delay_ms));

                    match AlsaDirectStream::new(plughw_device, config.sample_rate, config.channels)
                    {
                        Ok(stream) => {
                            log::info!(
                                "[ALSA Backend] ✓ plughw stream created on retry {} (after {}ms)",
                                i + 1,
                                delay_ms
                            );
                            return Some(Ok((stream, BitPerfectMode::PluginFallback)));
                        }
                        Err(e2) => {
                            log::warn!(
                                "[ALSA Backend] plughw retry {}/{} failed: {}",
                                i + 1,
                                retry_delays_ms.len(),
                                e2
                            );
                        }
                    }
                }
            }

            if matches!(error, AlsaDirectError::InvalidParams(_)) {
                // Hardware doesn't support this rate even via plughw.
                // Return None to let the player fall back to CPAL/rodio
                // which can resample (e.g. 176.4kHz → 88.2kHz).
                log::info!(
                    "[ALSA Backend] Hardware doesn't support {}Hz even via plughw, releasing device for CPAL fallback",
                    config.sample_rate
                );
                std::thread::sleep(std::time::Duration::from_millis(100));
                return None;
            }

            log::error!("[ALSA Backend] plughw fallback also failed: {}", e);
            Some(Err(format!(
                "Bit-perfect playback could not be established. hw failed, plughw failed: {}",
                e
            )))
        }
    }
}
