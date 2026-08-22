//! `AlsaBackend` construction and `/proc/asound`-driven device enumeration.

use super::device_id::is_known_pcm_id;
use super::device_list::{default_device_entry, push_card_devices};
use super::proc_cards::read_proc_asound_cards;
use super::AlsaBackend;
use crate::backend::{AudioDevice, BackendResult};
use rodio::cpal::traits::{DeviceTrait, HostTrait};
use std::collections::HashMap;

impl AlsaBackend {
    pub fn new() -> BackendResult<Self> {
        // Try to get ALSA host
        let available_hosts = rodio::cpal::available_hosts();

        // Check if ALSA is available
        // cpal 0.17 changed HostId::name() from "ALSA" to "Alsa" (uses stringify!)
        if !available_hosts
            .iter()
            .any(|h| h.name().eq_ignore_ascii_case("alsa"))
        {
            return Err("ALSA host not available on this system".to_string());
        }

        // Get ALSA host
        let host = rodio::cpal::host_from_id(
            available_hosts
                .into_iter()
                .find(|h| h.name().eq_ignore_ascii_case("alsa"))
                .ok_or("ALSA host not found".to_string())?,
        )
        .map_err(|e| format!("Failed to create ALSA host: {}", e))?;

        log::info!("[ALSA Backend] Initialized successfully");

        Ok(Self { host })
    }

    /// Enumerate ALSA devices using /proc/asound as PRIMARY source
    ///
    /// This approach ensures consistent device enumeration regardless of playback state.
    /// CPAL enumeration fails when devices are in exclusive mode, but /proc/asound
    /// always sees all devices.
    ///
    /// Architecture:
    /// 1. /proc/asound = PRIMARY source (always complete)
    /// 2. CPAL = OPTIONAL enrichment (sample rates only, may fail during playback)
    pub(super) fn enumerate_with_proc_descriptions(&self) -> BackendResult<Vec<AudioDevice>> {
        // Read all cards from /proc/asound (PRIMARY SOURCE)
        let cards = read_proc_asound_cards();

        log::info!("[ALSA Backend] /proc/asound found {} cards", cards.len());
        for card in &cards {
            log::debug!(
                "[ALSA Backend] Card {}: {} = {} ({} PCM devices)",
                card.number,
                card.short_name,
                card.long_name,
                card.pcm_playback_devices.len()
            );
        }

        // Build CPAL device map for sample rate enrichment (OPTIONAL - may be incomplete)
        let cpal_devices = self.build_cpal_device_map();
        log::debug!(
            "[ALSA Backend] CPAL found {} devices for enrichment",
            cpal_devices.len()
        );

        let mut devices = vec![default_device_entry(&cpal_devices)];

        // For each card, add relevant devices using STABLE IDs (card NAME, not number)
        for card in &cards {
            push_card_devices(&mut devices, card, &cpal_devices);
        }

        log::debug!("[ALSA Backend] Enumerated {} ALSA devices", devices.len());
        for (idx, dev) in devices.iter().enumerate() {
            log::debug!(
                "  [{}] {} - {} (max_rate: {:?}, rates: {:?})",
                idx,
                dev.name,
                dev.description.as_deref().unwrap_or("(default)"),
                dev.max_sample_rate,
                dev.supported_sample_rates
            );
        }

        Ok(devices)
    }

    /// Build a map of device_id -> CPAL Device for sample rate queries.
    /// This is OPTIONAL enrichment — devices may be missing if in exclusive use.
    ///
    /// Only the PCM name patterns we actually look up downstream are kept:
    /// `default`, `sysdefault:CARD=…`, and `{front,hdmi,iec958}:CARD=…,DEV=…`.
    /// Virtual PCMs (`dmix:`, `route:`, `surround51:` and the like) are
    /// dropped — we never query them, and letting them reach a later
    /// `supported_output_configs()` call just invites spurious libasound
    /// errors ("unable to open slave", "no matching channel map") on systems
    /// where PipeWire or another client holds the underlying hardware.
    pub(super) fn build_cpal_device_map(&self) -> HashMap<String, rodio::cpal::Device> {
        let mut map = HashMap::new();

        if let Ok(output_devices) = self.host.output_devices() {
            for device in output_devices {
                if let Ok(description) = device.description() {
                    let name = description.name().to_string();
                    if is_known_pcm_id(&name) {
                        map.insert(name, device);
                    }
                }
            }
        }

        map
    }
}
