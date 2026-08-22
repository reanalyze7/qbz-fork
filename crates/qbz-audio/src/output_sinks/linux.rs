use super::{cpal_device_name, OutputSinkInfo};

/// Enumerate the system's CPAL output devices.
///
/// On Linux the CPAL host is PipeWire/PulseAudio (rodio defaults to
/// CPAL's PipeWire host on modern distros); on macOS/Windows it is the
/// platform default. Output is shaped for the audio settings UI.
///
/// CRITICAL: The returned `name` is exactly the CPAL device name, so it
/// matches what the audio backend uses to re-open the device later. Do
/// NOT substitute a friendlier description for `name`.
pub fn list_output_sinks() -> Result<Vec<OutputSinkInfo>, String> {
    log::debug!("[qbz-audio] list_output_sinks (Linux, using CPAL)");

    use rodio::cpal::traits::{DeviceTrait, HostTrait};

    let host = rodio::cpal::default_host();

    let default_device_name = host
        .default_output_device()
        .and_then(|d| cpal_device_name(&d));

    log::debug!(
        "[qbz-audio] CPAL default device: {:?}",
        default_device_name
    );

    let sinks: Vec<OutputSinkInfo> = host
        .output_devices()
        .map_err(|e| format!("Failed to enumerate devices: {}", e))?
        .enumerate()
        .filter_map(|(idx, device)| {
            let name = match cpal_device_name(&device) {
                Some(name) => name,
                None => {
                    log::warn!("[qbz-audio]   [{}] Failed to get device description", idx);
                    return None;
                }
            };

            let is_default = default_device_name
                .as_ref()
                .map(|d| d == &name)
                .unwrap_or(false);

            // Same diagnostic logging as the legacy command, so log output
            // for the V2 command matches what users / support reports
            // already document.
            let configs_info = device
                .supported_output_configs()
                .ok()
                .map(|configs| {
                    let config_strs: Vec<String> = configs
                        .take(3)
                        .map(|c| format!("{}ch/{}Hz", c.channels(), c.max_sample_rate()))
                        .collect();
                    config_strs.join(", ")
                })
                .unwrap_or_else(|| "no configs".to_string());

            log::debug!(
                "[qbz-audio]   [{}] Device: '{}' (default: {}) - Configs: {}",
                idx,
                name,
                is_default,
                configs_info
            );

            // Use the CPAL name for both `name` and `description`: PipeWire
            // CPAL names are already user-friendly, and storing the same
            // value as `name` guarantees the saved id reopens correctly.
            Some(OutputSinkInfo {
                name: name.clone(),
                description: name,
                volume: None,
                is_default,
            })
        })
        .collect();

    // Collapse the per-PCM-plugin duplicates CPAL emits for a single output and
    // push the `null` discard sink to the end (shared with the System/JACK
    // backend enumeration — see device_filter).
    let sinks = crate::device_filter::retain_real_outputs(
        sinks,
        |s| s.name.as_str(),
        |s| s.description.as_str(),
    );

    log::debug!(
        "[qbz-audio] Found {} audio output devices via CPAL",
        sinks.len()
    );

    Ok(sinks)
}
