use super::{cpal_device_name, OutputSinkInfo};

/// Enumerate the system's CPAL output devices (macOS/Windows).
///
/// CPAL device names on these platforms are already descriptive enough
/// to display directly to the user.
pub fn list_output_sinks() -> Result<Vec<OutputSinkInfo>, String> {
    log::info!("[qbz-audio] list_output_sinks (non-Linux, using CPAL)");

    use rodio::cpal::traits::HostTrait;

    let host = rodio::cpal::default_host();

    let default_device_name = host
        .default_output_device()
        .and_then(|d| cpal_device_name(&d));

    let sinks: Vec<OutputSinkInfo> = host
        .output_devices()
        .map_err(|e| format!("Failed to enumerate devices: {}", e))?
        .filter_map(|device| {
            cpal_device_name(&device).map(|name| {
                let is_default = default_device_name
                    .as_ref()
                    .map(|d| d == &name)
                    .unwrap_or(false);
                OutputSinkInfo {
                    name: name.clone(),
                    description: name,
                    volume: None,
                    is_default,
                }
            })
        })
        .collect();

    let sinks = crate::device_filter::retain_real_outputs(
        sinks,
        |s| s.name.as_str(),
        |s| s.description.as_str(),
    );

    log::info!("[qbz-audio] Found {} audio output devices", sinks.len());
    Ok(sinks)
}
