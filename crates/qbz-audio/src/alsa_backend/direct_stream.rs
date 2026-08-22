//! `hw:`/`plughw:` direct-ALSA stream creation, bypassing CPAL.

use super::device_id::{extract_card_name_from_device, raw_open_ids};
use super::direct_stream_hw::{try_hw_attempt, HwOutcome};
use super::direct_stream_plughw::try_plughw_attempt;
use super::proc_rates::get_hw_supported_rates;
use super::AlsaBackend;
use crate::backend::{AlsaPlugin, BackendConfig, BitPerfectMode};
use crate::AlsaDirectStream;

impl AlsaBackend {
    /// Try to create direct ALSA stream for hw: devices (bypasses CPAL)
    /// Returns None if device is not a hw: device (should use CPAL instead)
    ///
    /// Implements controlled fallback:
    /// 1. Try direct hw access first
    /// 2. If format unsupported, try plughw (format conversion only, no resampling)
    /// 3. Abort on other errors (busy, permissions, etc.)
    pub fn try_create_direct_stream(
        &self,
        config: &BackendConfig,
    ) -> Option<Result<(AlsaDirectStream, BitPerfectMode), String>> {
        let device_id = config.device_id.as_ref()?;

        // Only use direct ALSA for hw:/plughw:/front: devices
        if !AlsaDirectStream::is_hw_device(device_id) {
            log::info!(
                "[ALSA Backend] Device '{}' is not hw:/plughw:/front:, using CPAL",
                device_id
            );
            return None;
        }

        // Determine the base device path for hw/plughw attempts. Aliased ids
        // (front:CARD=…) are opened through their raw hw:/plughw: forms: the
        // alias itself may be undeclared for the selected DEV (snd-aloop only
        // declares `front` for DEV=0) and needlessly routes through alsa-lib
        // plugins, while the raw ids open the kernel PCM directly (#641).
        let (hw_device, plughw_device) = if let Some(ids) = raw_open_ids(device_id) {
            ids
        } else if device_id.starts_with("hw:") {
            (device_id.to_string(), device_id.replace("hw:", "plughw:"))
        } else if device_id.starts_with("plughw:") {
            // Already plughw, try it directly
            (device_id.replace("plughw:", "hw:"), device_id.to_string())
        } else {
            (device_id.to_string(), format!("plug:{}", device_id))
        };

        // Pre-check: read /proc/asound/cardN/stream0 to verify rate support
        // before opening the PCM device. This avoids the "device busy" issue
        // where ALSA Direct opens the device, fails to configure the rate, and
        // leaves the device in a state CPAL can't open afterwards.
        //
        // When the rate is unsupported, we skip the hw: attempt and fall through
        // to the plughw: path below, which lets ALSA auto-resample in kernel/
        // userspace. Falling back to CPAL is not an option because CPAL's device
        // enumeration does not reliably expose raw hw: devices by the same name
        // the UI stored (front:CARD=X,DEV=Y vs hw:CARD=X,DEV=Y), and the lookup
        // fails with a misleading "Device not found" error. See issue #288.
        let mut hw_rate_unsupported = false;
        if let Some(card_name) = extract_card_name_from_device(device_id) {
            if let Some(hw_rates) = get_hw_supported_rates(&card_name) {
                if !hw_rates.contains(&config.sample_rate) {
                    log::info!(
                        "[ALSA Backend] Hardware rates for '{}': {:?}. {}Hz not supported natively — falling back to plughw for software resample",
                        card_name, hw_rates, config.sample_rate
                    );
                    hw_rate_unsupported = true;
                } else {
                    log::info!(
                        "[ALSA Backend] Hardware confirms support for {}Hz (card '{}', rates: {:?})",
                        config.sample_rate,
                        card_name,
                        hw_rates
                    );
                }
            }
        }

        // Respect ALSA plugin selection from settings. When /proc/asound already
        // told us the hw device won't accept the rate, skip straight to plughw
        // even if the user prefers Hw — plughw with resample is the only path
        // that produces sound.
        let try_hw_first = match config.alsa_plugin {
            Some(AlsaPlugin::Hw) => !hw_rate_unsupported,
            Some(AlsaPlugin::PlugHw) => false, // Skip hw, go directly to plughw
            Some(AlsaPlugin::Pcm) => {
                log::info!("[ALSA Backend] PCM mode selected, not using direct ALSA");
                return None; // Use CPAL instead
            }
            None => !hw_rate_unsupported, // Default: try hw first if rate is supported
        };

        if try_hw_first {
            match try_hw_attempt(&hw_device, config) {
                HwOutcome::Return(result) => return result,
                HwOutcome::FallThrough => {}
            }
        }

        // Try plughw fallback (format conversion only)
        try_plughw_attempt(&plughw_device, config)
    }
}
