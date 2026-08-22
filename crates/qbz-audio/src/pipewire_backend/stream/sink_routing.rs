//! Resolves and (optionally) switches the PipeWire/PulseAudio default sink
//! for the stream about to be opened.

use crate::backend::BackendConfig;
use std::process::Command;

/// Temporarily set default sink to target (if specified).
/// We DON'T restore it - let the user's system keep the selected device as default.
/// This is actually the expected behavior: when you select a device, it becomes the default.
/// When skip_sink_switch is true, skip this entirely to preserve external routing (JACK/qjackctl).
///
/// Returns the target sink (the `config.device_id`, unchanged) for callers that
/// still need it downstream (locked-mode `PIPEWIRE_NODE` routing).
pub(super) fn switch_default_sink_if_needed(config: &BackendConfig) -> Option<String> {
    let target_sink = config.device_id.clone();

    if config.skip_sink_switch {
        log::info!("[PipeWire Backend] Skipping set-default-sink (skip_sink_switch enabled)");
    } else if let Some(sink_name) = &target_sink {
        log::info!("[PipeWire Backend] Setting default sink to: {}", sink_name);

        let set_result = Command::new("pactl")
            .args(["set-default-sink", sink_name])
            .output();

        match set_result {
            Ok(output) if output.status.success() => {
                log::info!("[PipeWire Backend] Default sink set to {}", sink_name);
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                log::warn!("[PipeWire Backend] Failed to set default sink: {}", stderr);
            }
            Err(e) => {
                log::warn!(
                    "[PipeWire Backend] Error executing pactl set-default-sink: {}",
                    e
                );
            }
        }

        // Wait for PipeWire to process the default sink change
        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    target_sink
}

/// Check if the DAC supports the requested sample rate.
/// Query via /proc/asound/ (USB DACs list discrete supported rates).
/// If unsupported, fall back to the nearest rate in the same family
/// (e.g., 176.4kHz → 88.2kHz). Rodio resamples from track rate to
/// stream rate automatically.
///
/// Resolves the effective sink: the explicit target, else whatever
/// `pactl get-default-sink` currently reports.
pub(super) fn resolve_effective_sink(target_sink: &Option<String>) -> Option<String> {
    target_sink.clone().or_else(|| {
        Command::new("pactl")
            .args(["get-default-sink"])
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    String::from_utf8(o.stdout)
                        .ok()
                        .map(|s| s.trim().to_string())
                } else {
                    None
                }
            })
    })
}
