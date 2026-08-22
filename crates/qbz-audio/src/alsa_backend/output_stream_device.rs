//! Device resolution (with the `hw:CARD=` alias-fallback dance) for
//! `AlsaBackend::create_output_stream`, split out of that function purely to
//! stay under the per-file line budget — behavior is unchanged.

use super::device_id::{build_hw_fallback_id, is_card_present_in_proc};
use crate::backend::BackendConfig;
use rodio::cpal::traits::{DeviceTrait, HostTrait};

/// Find the device by name/id.
///
/// If /proc/asound shows this device exists but CPAL's enumeration
/// cannot match it, the app stored a name format CPAL does not expose
/// (e.g. front:CARD=X,DEV=Y when CPAL only yields hw:CARD=X,DEV=Y for
/// the raw device). Surface that distinction in the error so users
/// don't chase ghosts wondering why their DAC "disappeared".
pub(super) fn resolve_output_device(
    host: &rodio::cpal::Host,
    config: &BackendConfig,
) -> Result<rodio::cpal::Device, String> {
    if let Some(device_id) = &config.device_id {
        log::info!("[ALSA Backend] Looking for device: {}", device_id);
        let primary = host
            .output_devices()
            .map_err(|e| format!("Failed to enumerate devices: {}", e))?
            .find(|d| {
                d.description()
                    .ok()
                    .map(|desc| desc.name() == device_id.as_str())
                    .unwrap_or(false)
            });

        // Fallback: if the primary alias didn't resolve but /proc/asound
        // shows the card is present, retry with the raw hw:CARD=<name>,
        // DEV=<n> PCM. The kernel driver always exposes that for any
        // registered card, which covers minimal distro configs where
        // iec958:/hdmi:/front: aliases aren't declared in asound.conf
        // (issue #331 — HifiBerry Digi2 Pro on Raspberry Pi OS).
        let resolved = match primary {
            Some(d) => Some(d),
            None => match build_hw_fallback_id(device_id) {
                Some(hw_id) if is_card_present_in_proc(device_id) => {
                    log::warn!(
                        "[ALSA Backend] '{}' not resolvable by ALSA (alias likely missing in asound.conf); trying fallback '{}'",
                        device_id,
                        hw_id
                    );
                    let found = host
                        .output_devices()
                        .map_err(|e| format!("Failed to enumerate devices: {}", e))?
                        .find(|d| {
                            d.description()
                                .ok()
                                .map(|desc| desc.name() == hw_id.as_str())
                                .unwrap_or(false)
                        });
                    if found.is_some() {
                        log::info!("[ALSA Backend] Using fallback device: {}", hw_id);
                    }
                    found
                }
                _ => None,
            },
        };

        resolved.ok_or_else(|| {
            let proc_found = is_card_present_in_proc(device_id);
            if proc_found {
                format!(
                    "Device '{}' is present in /proc/asound but CPAL cannot open it (usually a sample-rate/format mismatch — track rate {}Hz, or an ALSA name format mismatch). Try the plughw plugin in audio settings.",
                    device_id, config.sample_rate
                )
            } else {
                format!(
                    "Device '{}' not found by the ALSA backend (disconnected, renamed, or handled by another app)",
                    device_id
                )
            }
        })
    } else {
        log::info!("[ALSA Backend] Using default device");
        host.default_output_device()
            .ok_or_else(|| "No default ALSA device available".to_string())
    }
}
