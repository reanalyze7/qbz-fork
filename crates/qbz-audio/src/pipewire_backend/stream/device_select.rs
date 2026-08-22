//! CPAL host/device selection for PipeWire/PulseAudio, and the locked-mode
//! `PIPEWIRE_NODE` env-var routing guard (Tier 2a, issue #263).

use super::super::PwNodeEnvGuard;
use crate::backend::BackendConfig;
use rodio::cpal::traits::{DeviceTrait, HostTrait};

/// Find a CPAL device backed by PulseAudio/PipeWire.
/// Newer CPAL description().name() returns friendly labels like
/// "PipeWire Sound Server" instead of raw ids ("pipewire"/"pulse").
pub(super) fn select_cpal_device() -> Result<rodio::cpal::Device, String> {
    // Create a NEW host (will use current default sink)
    log::info!("[PipeWire Backend] Creating fresh CPAL host...");
    let fresh_host = rodio::cpal::default_host();

    let mut best_device: Option<rodio::cpal::Device> = None;
    let mut best_score: u8 = 0;
    let mut available_output_devices: Vec<String> = Vec::new();

    for device in fresh_host
        .output_devices()
        .map_err(|e| format!("Failed to enumerate CPAL devices: {}", e))?
    {
        let device_name = device
            .description()
            .map(|desc| desc.name().to_string())
            .unwrap_or_else(|_| "unknown".to_string());
        let device_name_lower = device_name.to_ascii_lowercase();
        available_output_devices.push(device_name.clone());

        let score = if device_name_lower == "pipewire" || device_name_lower == "pulse" {
            3
        } else if device_name_lower.contains("pipewire sound server")
            || device_name_lower.contains("pulseaudio sound server")
        {
            2
        } else if device_name_lower.contains("pipewire") || device_name_lower.contains("pulseaudio")
        {
            1
        } else {
            0
        };

        if score > best_score {
            best_score = score;
            best_device = Some(device);
        }
    }

    let device = best_device.ok_or_else(|| {
        format!(
            "Could not find 'pulse' or 'pipewire' CPAL device. Is PulseAudio/PipeWire running? Available output devices: {:?}",
            available_output_devices
        )
    })?;

    let device_name = device
        .description()
        .map(|desc| desc.name().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    log::info!("[PipeWire Backend] Using CPAL device: {}", device_name);

    Ok(device)
}

/// Tier 2a (#263): in locked mode (skip_sink_switch) QBZ does NOT steal the
/// system default sink, so route THIS stream to the selected sink via the
/// pipewire-ALSA plugin's PIPEWIRE_NODE env — it targets that node WITHOUT
/// changing the system default. The returned guard restores the prior env
/// value when dropped (kept alive until the end of stream creation).
#[cfg(target_os = "linux")]
pub(super) fn lock_pipewire_node(
    config: &BackendConfig,
    target_sink: &Option<String>,
) -> Option<PwNodeEnvGuard> {
    if !config.skip_sink_switch {
        return None;
    }
    target_sink.as_ref().map(|sink| {
        let prev = std::env::var("PIPEWIRE_NODE").ok();
        std::env::set_var("PIPEWIRE_NODE", sink);
        log::info!(
            "[PipeWire Backend] Targeting sink '{}' via PIPEWIRE_NODE (locked mode, default unchanged)",
            sink
        );
        PwNodeEnvGuard(prev)
    })
}
