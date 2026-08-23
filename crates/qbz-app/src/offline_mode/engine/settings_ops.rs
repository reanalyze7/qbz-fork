use std::sync::atomic::Ordering;

use super::OfflineModeEngine;
use crate::offline_mode::store::OfflineModeSettings;
use crate::offline_mode::OfflineStatus;
use qbz_audio::settings::AudioSettingsStore;

impl OfflineModeEngine {
    /// Read the persisted settings (Settings view).
    pub fn settings(&self) -> Result<OfflineModeSettings, String> {
        let guard = self
            .store
            .lock()
            .map_err(|e| format!("offline store lock poisoned: {}", e))?;
        let store = guard.as_ref().ok_or("No active session")?;
        store.get_settings()
    }

    /// Persist the network-folders-in-manual-offline policy flag.
    ///
    /// NOTE (2026-06-10): no UI calls this anymore — the Slint "Show Network
    /// Folder Content" toggle was removed when library visibility stopped
    /// depending on offline mode (owner verdict; see qbz-slint's
    /// NETWORK-FOLDER VISIBILITY note). Kept (pub, no dead-code warning in a
    /// lib crate) because the store column must stay Tauri-DB-compatible.
    pub fn set_show_network_folders(&self, enabled: bool) -> Result<(), String> {
        let guard = self
            .store
            .lock()
            .map_err(|e| format!("offline store lock poisoned: {}", e))?;
        let store = guard.as_ref().ok_or("No active session")?;
        store.set_show_network_folders_in_manual_offline(enabled)
    }

    /// Flip induced offline (Settings toggle). Always succeeds in either
    /// direction; persists the flag, handles the #279 snapshot/restore, then
    /// recomputes the mode (which flips the Qobuz gate).
    ///
    /// `audio` is best-effort: pre-login there may be no audio store yet.
    pub fn set_induced(
        &self,
        enabled: bool,
        audio: Option<&AudioSettingsStore>,
    ) -> Result<OfflineStatus, String> {
        let was = {
            let guard = self
                .store
                .lock()
                .map_err(|e| format!("offline store lock poisoned: {}", e))?;
            let store = guard.as_ref().ok_or("No active session")?;
            let was = store.get_settings()?.manual_offline_mode;
            store.set_manual_offline_mode(enabled)?;

            if let Some(audio_store) = audio {
                if enabled && !was {
                    // Entering: stash the current preference, force false.
                    if let Ok(settings) = audio_store.get_settings() {
                        let _ = store
                            .set_pre_offline_stream_first_track(Some(settings.stream_first_track));
                        if settings.stream_first_track {
                            let _ = audio_store.set_stream_first_track(false);
                            log::info!(
                                "[OfflineMode] stream_first_track snapshot=true; forced false while offline (#279)"
                            );
                        }
                    }
                } else if !enabled && was {
                    // Exiting: restore the stash, clear it.
                    if let Ok(Some(snapshot)) = store.get_pre_offline_stream_first_track() {
                        let _ = audio_store.set_stream_first_track(snapshot);
                        let _ = store.set_pre_offline_stream_first_track(None);
                        log::info!("[OfflineMode] stream_first_track restored to {} (#279)", snapshot);
                    }
                }
            }
            was
        };
        let _ = was;

        self.induced.store(enabled, Ordering::Relaxed);
        self.recompute();
        Ok(self.status())
    }
}
