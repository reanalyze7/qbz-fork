use std::sync::Arc;
use std::sync::Mutex;

use qbz_audio::{settings::AudioSettingsStore, AudioDiagnostic, AudioSettings, VisualizerTap};
use qbz_core::{FrontendAdapter, QbzCore};
use qbz_player::Player;

use super::AppRuntime;
use crate::runtime::RuntimeManager;
use crate::user_data::UserDataPaths;

impl<A: FrontendAdapter + Send + Sync + 'static> AppRuntime<A> {
    /// Build with explicit audio settings.
    ///
    /// Performs no disk or network access — used by tests and by shells that
    /// already have audio settings loaded.
    pub fn with_audio_settings(
        adapter: A,
        device_name: Option<String>,
        audio_settings: AudioSettings,
        visualizer_tap: Option<VisualizerTap>,
    ) -> Self {
        let diagnostic = AudioDiagnostic::new();
        let player = Player::new(device_name, audio_settings, visualizer_tap, diagnostic);
        let core = QbzCore::new(adapter, player);
        Self {
            core: Arc::new(core),
            runtime: Arc::new(RuntimeManager::new()),
            user_paths: UserDataPaths::new(),
            session: Mutex::new(None),
            visualizer_tap: None,
        }
    }

    /// Build, loading persisted audio settings from [`AudioSettingsStore`].
    ///
    /// Falls back to defaults when no settings are saved or the store cannot
    /// be opened. This mirrors the recipe in the Tauri `CoreBridge::new`.
    /// It does not touch the network — call [`AppRuntime::init`] for that.
    pub fn new(adapter: A) -> Self {
        let (device_name, audio_settings) = AudioSettingsStore::new()
            .ok()
            .and_then(|store| {
                store
                    .get_settings()
                    .ok()
                    .map(|settings| (settings.output_device.clone(), settings))
            })
            .unwrap_or_else(|| {
                log::info!("[AppRuntime] No saved audio settings, using defaults");
                (None, AudioSettings::default())
            });
        Self::with_audio_settings(adapter, device_name, audio_settings, None)
    }

    /// Build like [`AppRuntime::new`], but also wire a [`VisualizerTap`] into the
    /// player and retain it so the shell can start the frontend-agnostic FFT
    /// producer ([`qbz_audio::visualizer::spawn_visualizer_thread`]) and toggle
    /// capture via the tap's `set_enabled`. Used by the Slint shell for the
    /// ImmersiveView audio visualizers. The tap starts disabled, so it adds no
    /// runtime cost until the immersive view enables it.
    pub fn with_visualizer(adapter: A) -> Self {
        let (device_name, audio_settings) = AudioSettingsStore::new()
            .ok()
            .and_then(|store| {
                store
                    .get_settings()
                    .ok()
                    .map(|settings| (settings.output_device.clone(), settings))
            })
            .unwrap_or_else(|| {
                log::info!("[AppRuntime] No saved audio settings, using defaults");
                (None, AudioSettings::default())
            });
        let tap = VisualizerTap::new();
        let mut rt =
            Self::with_audio_settings(adapter, device_name, audio_settings, Some(tap.clone()));
        rt.visualizer_tap = Some(tap);
        rt
    }

    /// The visualizer tap, if this runtime was built with [`AppRuntime::with_visualizer`].
    pub fn visualizer_tap(&self) -> Option<&VisualizerTap> {
        self.visualizer_tap.as_ref()
    }

    /// Initialize the core (extracts Qobuz bundle tokens).
    ///
    /// Best-effort and offline-tolerant: a network failure here leaves the
    /// core usable for local/offline playback, matching [`QbzCore::init`].
    pub async fn init(&self) -> Result<(), String> {
        self.core.init().await.map_err(|e| e.to_string())
    }

    /// The orchestrator. Shells reach catalog, playback, queue, and auth
    /// functionality through this handle.
    pub fn core(&self) -> &Arc<QbzCore<A>> {
        &self.core
    }

    /// The framework-agnostic runtime state machine.
    pub fn runtime(&self) -> &Arc<RuntimeManager> {
        &self.runtime
    }
}
