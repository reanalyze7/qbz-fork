//! Raw `hw:` attempt (with busy-retry backoff) for
//! `AlsaBackend::try_create_direct_stream`, split out of that function
//! purely to stay under the per-file line budget — behavior is unchanged.

use super::pipewire_suspend::suspend_default_sink_for_exclusive;
use crate::backend::{AlsaDirectError, BackendConfig, BitPerfectMode};
use crate::AlsaDirectStream;

/// Outcome of the raw `hw:` attempt: either a final result to return from
/// `try_create_direct_stream`, or a signal to fall through to the `plughw:`
/// attempt.
pub(super) enum HwOutcome {
    Return(Option<Result<(AlsaDirectStream, BitPerfectMode), String>>),
    FallThrough,
}

pub(super) fn try_hw_attempt(hw_device: &str, config: &BackendConfig) -> HwOutcome {
    log::info!(
        "[ALSA Backend] Attempting DIRECT hw stream: {} ({}Hz, {}ch)",
        hw_device,
        config.sample_rate,
        config.channels
    );

    match AlsaDirectStream::new(hw_device, config.sample_rate, config.channels) {
        Ok(stream) => {
            log::info!("[ALSA Backend] ✓ Direct hw stream created successfully");
            HwOutcome::Return(Some(Ok((stream, BitPerfectMode::DirectHardware))))
        }
        Err(e) => {
            let error = AlsaDirectError::from_alsa_error(&e);
            log::warn!("[ALSA Backend] hw attempt failed: {}", error);

            if matches!(error, AlsaDirectError::DeviceBusy(_)) {
                // Device busy: either our own previous PCM handle is still
                // releasing (race on fast track skip) or PipeWire holds it.
                // Retry with progressive backoff before giving up.
                log::info!("[ALSA Backend] Device busy — retrying with backoff");

                // Try suspending PipeWire once (covers PipeWire-held case).
                // Records the sink so it is resumed on stop/teardown (#263).
                suspend_default_sink_for_exclusive();

                let retry_delays_ms = [50, 100, 200, 400, 800];
                for (i, delay_ms) in retry_delays_ms.iter().enumerate() {
                    std::thread::sleep(std::time::Duration::from_millis(*delay_ms));

                    match AlsaDirectStream::new(hw_device, config.sample_rate, config.channels) {
                        Ok(stream) => {
                            log::info!(
                                "[ALSA Backend] ✓ Direct hw stream created on retry {} (after {}ms)",
                                i + 1,
                                delay_ms
                            );
                            return HwOutcome::Return(Some(Ok((
                                stream,
                                BitPerfectMode::DirectHardware,
                            ))));
                        }
                        Err(e2) => {
                            log::warn!(
                                "[ALSA Backend] Retry {}/{} failed: {}",
                                i + 1,
                                retry_delays_ms.len(),
                                e2
                            );
                        }
                    }
                }

                log::error!(
                    "[ALSA Backend] Cannot acquire device after {} retries",
                    retry_delays_ms.len()
                );
                return HwOutcome::Return(Some(Err(format!(
                    "ALSA Direct failed: {}. Device may be in use or inaccessible.",
                    error
                ))));
            }

            if matches!(error, AlsaDirectError::InvalidParams(_)) {
                // Hardware doesn't support this rate/format natively.
                // Return None to let the player fall back to CPAL/rodio
                // which can resample (e.g. 176.4kHz → 88.2kHz).
                // Brief delay: ALSA Direct opened the PCM (then failed to configure it).
                // The kernel needs a moment to fully release the device before CPAL can open it.
                log::info!(
                    "[ALSA Backend] Hardware doesn't support {}Hz natively, releasing device for CPAL fallback",
                    config.sample_rate
                );
                std::thread::sleep(std::time::Duration::from_millis(100));
                return HwOutcome::Return(None);
            }

            if !error.allows_plughw_fallback() {
                // Non-recoverable error (permissions, etc.)
                log::error!("[ALSA Backend] Cannot fallback - error type: {:?}", error);
                return HwOutcome::Return(Some(Err(format!(
                    "ALSA Direct failed: {}. Device may be in use or inaccessible.",
                    error
                ))));
            }

            log::info!("[ALSA Backend] Format unsupported on hw, trying plughw fallback...");
            HwOutcome::FallThrough
        }
    }
}
