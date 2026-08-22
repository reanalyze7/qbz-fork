use super::*;

impl Player {
    /// DoP/native resolution (DSD plan Phase 2/3): user opt-in + ALSA direct
    /// backend + stereo + carrier rate supported by the device. Returns
    /// `Some(result)` when a direct-mode `AudioCommand` was sent (caller
    /// must have already dropped its demuxer handle); `None` means fall
    /// through to the universal DSD->PCM conversion path.
    #[cfg(target_os = "linux")]
    pub(super) fn try_direct_dsd(
        &self,
        info: &qbz_dsd::DsdStreamInfo,
        path: &std::path::Path,
        track_id: u64,
    ) -> Option<Result<(), String>> {
        let resolved = self
            .audio_settings
            .lock()
            .ok()
            .map(|s| {
                (
                    s.dsd_mode.clone(),
                    matches!(s.backend_type, Some(qbz_audio::AudioBackendType::Alsa)),
                    s.output_device.clone(),
                )
            })
            .unwrap_or(("convert".to_string(), false, None));

        let (mode, true, Some(device)) = resolved else {
            return None;
        };

        if info.channels == 2 && mode != "convert" {
            if mode == "native" {
                log::info!(
                    "Player: DSD track {} — {} via NATIVE DSD",
                    track_id,
                    qbz_dsd::dsd_label(info.dsd_rate)
                );
                return Some(
                    self.tx
                        .send(AudioCommand::PlayDsdNative {
                            path: path.to_path_buf(),
                            track_id,
                        })
                        .map_err(|e| format!("Failed to send native DSD play command: {}", e)),
                );
            }
            let carrier = qbz_dsd::dop_carrier_rate(info.dsd_rate);
            let rate_ok = qbz_audio::alsa_backend::get_device_supported_rates(&device)
                .map(|r| r.contains(&carrier))
                .unwrap_or(true);
            if rate_ok {
                log::info!(
                    "Player: DSD track {} — {} via DoP ({} Hz carrier)",
                    track_id,
                    qbz_dsd::dsd_label(info.dsd_rate),
                    carrier
                );
                return Some(
                    self.tx
                        .send(AudioCommand::PlayDsdDop {
                            path: path.to_path_buf(),
                            track_id,
                        })
                        .map_err(|e| format!("Failed to send DoP play command: {}", e)),
                );
            }
            log::info!(
                "Player: DoP selected but device lacks the {} Hz carrier — converting to PCM",
                carrier
            );
        } else if info.channels != 2 && mode != "convert" {
            log::info!(
                "Player: {} selected but track has {} channels — downmix-converting to PCM",
                mode,
                info.channels
            );
        }
        None
    }
}
