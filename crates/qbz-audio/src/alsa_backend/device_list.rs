//! Device-list assembly helpers for `enumerate_with_proc_descriptions`,
//! split out of the orchestrating function purely to stay under the
//! per-file line budget — behavior is unchanged.

use super::proc_cards::ProcCardInfo;
use super::sample_rates::get_supported_sample_rates;
use crate::backend::AudioDevice;
use std::collections::HashMap;

/// Build the "system default" `AudioDevice` entry.
pub(super) fn default_device_entry(
    cpal_devices: &HashMap<String, rodio::cpal::Device>,
) -> AudioDevice {
    let default_sample_rates = cpal_devices
        .get("default")
        .and_then(|d| get_supported_sample_rates(d));
    let default_max_rate = default_sample_rates
        .as_ref()
        .and_then(|rates| rates.iter().max().copied());

    AudioDevice {
        id: "default".to_string(),
        name: "default".to_string(),
        description: None, // Frontend shows "System Default"
        is_default: true,
        max_sample_rate: default_max_rate.or(Some(384000)),
        supported_sample_rates: default_sample_rates,
        device_bus: None,
        is_hardware: false,
    }
}

/// Push the `sysdefault:CARD=…` entry and any PCM-specific (`front:`,
/// `iec958:`, `hdmi:`) entries for one card onto `devices`.
///
/// Skips cards with no PCM playback devices — `/proc/asound/cards` lists
/// every registered sound card, including capture-only hardware (USB
/// webcams, microphones, HDMI-audio-less capture devices), which would
/// otherwise show up as a bogus selectable "output device".
pub(super) fn push_card_devices(
    devices: &mut Vec<AudioDevice>,
    card: &ProcCardInfo,
    cpal_devices: &HashMap<String, rodio::cpal::Device>,
) {
    if card.pcm_playback_devices.is_empty() {
        log::debug!(
            "[ALSA Backend] Skipping capture-only card {} ({}) — no playback PCMs",
            card.number,
            card.short_name
        );
        return;
    }

    // Add sysdefault:CARD=name (card default with software mixing)
    let sysdefault_id = format!("sysdefault:CARD={}", card.short_name);
    let sysdefault_rates = cpal_devices
        .get(&sysdefault_id)
        .and_then(|d| get_supported_sample_rates(d));

    devices.push(AudioDevice {
        id: sysdefault_id.clone(),
        name: sysdefault_id.clone(),
        description: Some(format!("{}, {}", card.long_name, sysdefault_id)),
        is_default: false,
        max_sample_rate: sysdefault_rates
            .as_ref()
            .and_then(|r| r.iter().max().copied())
            .or(Some(192000)),
        supported_sample_rates: sysdefault_rates,
        device_bus: None,
        is_hardware: false, // sysdefault uses dmix
    });

    // Add PCM-specific devices (front:, iec958:, hdmi:)
    for pcm in &card.pcm_playback_devices {
        // Determine device type based on PCM name
        let device_prefix = if pcm.name.to_lowercase().contains("hdmi") {
            "hdmi"
        } else if pcm.name.to_lowercase().contains("iec958")
            || pcm.name.to_lowercase().contains("spdif")
            || pcm.name.to_lowercase().contains("s/pdif")
        {
            "iec958"
        } else {
            "front" // Default to front: for analog/USB audio
        };

        let device_id = format!(
            "{}:CARD={},DEV={}",
            device_prefix, card.short_name, pcm.device_num
        );

        // Skip if already added (shouldn't happen, but be safe)
        if devices.iter().any(|d| d.id == device_id) {
            continue;
        }

        // Try to get sample rates from CPAL (may fail if device is busy)
        let sample_rates = cpal_devices
            .get(&device_id)
            .and_then(|d| get_supported_sample_rates(d));
        let max_rate = sample_rates
            .as_ref()
            .and_then(|r| r.iter().max().copied())
            .or(Some(384000)); // Assume high capability if CPAL unavailable

        devices.push(AudioDevice {
            id: device_id.clone(),
            name: device_id.clone(),
            description: Some(format!("{}, {}", card.long_name, device_id)),
            is_default: false,
            max_sample_rate: max_rate,
            supported_sample_rates: sample_rates,
            device_bus: None,
            is_hardware: true,
        });
    }
}
