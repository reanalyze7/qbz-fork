//! Exclusive-mode (Hog Mode) stream creation, for macOS only.

use super::super::cpal_default::CpalDefaultBackend;
use super::super::device_config::{BackendConfig, BackendResult};
use rodio::MixerDeviceSink;

impl CpalDefaultBackend {
    pub(in super::super) fn create_output_stream_with_exclusive_guard_impl(
        &self,
        config: &BackendConfig,
    ) -> BackendResult<(
        MixerDeviceSink,
        Option<crate::coreaudio_direct::CoreAudioExclusiveGuard>,
    )> {
        if !config.exclusive_mode {
            return self.create_output_stream(config).map(|sink| (sink, None));
        }

        // Resolve the target device ONCE before acquiring Hog Mode and pin its
        // name into the config we hand to `create_output_stream`. Without this,
        // the rate-switch and CPAL-stream paths inside `create_output_stream`
        // re-resolve the System Default by name — and macOS reassigns the
        // System Default the moment we hog the previous one (so other apps
        // still get audio). The result is Hog Mode held on device A while the
        // stream is created on device B, which on a multi-device machine
        // (e.g. an external DAC alongside built-in speakers) leaves the DAC
        // hogged-but-unused and audio routed to a device that may not even
        // support the requested sample rate.
        let device_id =
            crate::coreaudio_direct::resolve_output_device_id(config.device_id.as_deref())?;
        let device_name = crate::coreaudio_direct::get_device_name(device_id)?;
        let guard = crate::coreaudio_direct::CoreAudioExclusiveGuard::acquire(device_id)?;

        let mut effective_config = config.clone();
        effective_config.device_id = Some(device_name);

        self.create_output_stream(&effective_config)
            .map(|sink| (sink, Some(guard)))
    }
}
