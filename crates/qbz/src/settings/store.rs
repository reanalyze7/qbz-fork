//! Shared settings state: the two domain stores, the dropdown index maps,
//! and the lock-and-call helpers every other submodule closes over.

use std::sync::{Arc, Mutex};

use qbz_app::settings::playback::{PlaybackPreferencesState, PlaybackPreferencesStore};
use qbz_audio::backend::AudioBackendType;
use qbz_audio::settings::{AudioSettingsState, AudioSettingsStore};

/// What a persisted audio change requires of the live `Player`.
pub(super) enum Apply {
    /// Not a player-applied setting — nothing to apply.
    None,
    /// Settings struct refresh only (gapless, fallback, stream-*, ...).
    Reload,
    /// Routing-critical — also re-init the output device.
    Reinit,
}

/// Index -> value maps the dropdown callbacks resolve against. The label
/// lists live in `SettingsState`; these are the parallel value lists.
#[derive(Default)]
pub(super) struct SettingsMaps {
    pub(super) backends: Vec<AudioBackendType>,
    /// Device ids, parallel to `SettingsState.devices` labels. An empty
    /// id is the "System default" entry (`output_device = None`).
    pub(super) devices: Vec<String>,
}

/// Owns the settings stores and the dropdown index maps.
pub struct SettingsCtx {
    pub(super) audio: AudioSettingsState,
    pub(super) playback: PlaybackPreferencesState,
    pub(super) maps: Mutex<SettingsMaps>,
}

impl SettingsCtx {
    /// Open both domain stores at the shared global path. A store that
    /// fails to open degrades to an empty (no-op) handle rather than
    /// aborting.
    pub fn open() -> Arc<Self> {
        let audio = AudioSettingsState::new().unwrap_or_else(|e| {
            log::warn!("[qbz-slint] audio settings store unavailable: {e}");
            AudioSettingsState::new_empty()
        });
        let playback = PlaybackPreferencesState::new().unwrap_or_else(|e| {
            log::warn!("[qbz-slint] playback preferences store unavailable: {e}");
            PlaybackPreferencesState::new_empty()
        });
        Arc::new(Self {
            audio,
            playback,
            maps: Mutex::new(SettingsMaps::default()),
        })
    }

    /// A handle to the playback-preferences store sharing the same
    /// underlying SQLite connection. The Queue controller uses it to read
    /// and toggle the autoplay (infinite-play) mode so the sidebar's
    /// infinite-play button stays in step with the Playback settings.
    pub fn playback_prefs(&self) -> PlaybackPreferencesState {
        PlaybackPreferencesState {
            store: std::sync::Arc::clone(&self.playback.store),
        }
    }
}

pub(super) fn with_audio<T>(
    audio: &AudioSettingsState,
    f: impl FnOnce(&AudioSettingsStore) -> Result<T, String>,
) -> Result<T, String> {
    let guard = audio
        .store
        .lock()
        .map_err(|_| "audio settings lock poisoned".to_string())?;
    let store = guard
        .as_ref()
        .ok_or_else(|| "audio settings store not open".to_string())?;
    f(store)
}

pub(super) fn with_playback<T>(
    playback: &PlaybackPreferencesState,
    f: impl FnOnce(&PlaybackPreferencesStore) -> Result<T, String>,
) -> Result<T, String> {
    let guard = playback
        .store
        .lock()
        .map_err(|_| "playback preferences lock poisoned".to_string())?;
    let store = guard
        .as_ref()
        .ok_or_else(|| "playback preferences store not open".to_string())?;
    f(store)
}
