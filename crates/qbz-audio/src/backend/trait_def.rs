//! The `AudioBackend` trait: the contract every audio backend implements.

use super::types::{AudioBackendType, AudioDevice, BackendConfig, BackendResult};
use rodio::MixerDeviceSink;

/// Audio backend trait
///
/// All audio backends must implement this trait to provide
/// a consistent interface for device enumeration and stream creation.
pub trait AudioBackend: Send + Sync {
    /// Get the backend type
    fn backend_type(&self) -> AudioBackendType;

    /// Enumerate available audio devices for this backend
    fn enumerate_devices(&self) -> BackendResult<Vec<AudioDevice>>;

    /// Create an output stream for the given configuration
    fn create_output_stream(&self, config: &BackendConfig) -> BackendResult<MixerDeviceSink>;

    /// Create an output stream and optionally return a platform exclusive-mode guard.
    /// Most backends do not need a guard; macOS CoreAudio uses this to keep Hog Mode
    /// owned for the lifetime of the stream.
    fn create_output_stream_with_exclusive_guard(
        &self,
        config: &BackendConfig,
    ) -> BackendResult<(
        MixerDeviceSink,
        Option<crate::coreaudio_direct::CoreAudioExclusiveGuard>,
    )> {
        self.create_output_stream(config).map(|sink| (sink, None))
    }

    /// Check if this backend is available on the current system
    fn is_available(&self) -> bool;

    /// Get a description of this backend for UI display
    fn description(&self) -> &'static str;

    /// Downcast to concrete type (for ALSA Direct stream creation)
    fn as_any(&self) -> &dyn std::any::Any;
}
