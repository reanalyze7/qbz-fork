use super::*;

pub(crate) fn cpal_device_name(device: &rodio::cpal::Device) -> Option<String> {
    device
        .description()
        .ok()
        .map(|description| description.name().to_string())
}

/// Output stream type - either rodio or ALSA Direct.
///
/// On macOS, the Rodio variant carries an optional `exclusive_guard`
/// whose `Drop` releases CoreAudio Hog Mode and is otherwise inert —
/// so exclusive and shared modes share a single code path.
pub(crate) enum StreamType {
    Rodio {
        sink: MixerDeviceSink,
        /// Holds CoreAudio Hog Mode for the lifetime of the stream.
        /// Load-bearing via `Drop`; reads happen through pattern matches
        /// (e.g., `set_coreaudio_hardware_volume`).
        #[cfg(target_os = "macos")]
        exclusive_guard: Option<qbz_audio::CoreAudioExclusiveGuard>,
    },
    #[cfg(target_os = "linux")]
    AlsaDirect(Arc<qbz_audio::AlsaDirectStream>),
    /// Native JACK output (#263 Tier 3). QBZ as a JACK client with stable ports;
    /// NOT bit-perfect (resampled to the graph rate).
    #[cfg(target_os = "linux")]
    Jack(Arc<qbz_audio::JackStream>),
}

impl StreamType {
    /// Construct a shared-mode Rodio stream (no exclusive guard).
    pub(crate) fn rodio(sink: MixerDeviceSink) -> Self {
        StreamType::Rodio {
            sink,
            #[cfg(target_os = "macos")]
            exclusive_guard: None,
        }
    }

    /// Apply the volume to CoreAudio hardware if the device supports it.
    ///
    /// Returns `true` only when the hardware accepted the change so the
    /// caller can pin the software stream to unity gain. Returns `false`
    /// for shared mode and for knob-only DACs (no settable volume
    /// property), letting the caller fall back to software volume.
    #[cfg(target_os = "macos")]
    fn set_coreaudio_hardware_volume(&self, volume: f32) -> bool {
        match self {
            StreamType::Rodio {
                exclusive_guard: Some(guard),
                ..
            } => match guard.set_hardware_volume(volume) {
                Ok(()) => true,
                Err(e) => {
                    log::warn!(
                        "[CoreAudio] Hardware volume failed; falling back to software: {}",
                        e
                    );
                    false
                }
            },
            _ => false,
        }
    }

    /// Actual output stream rate. On macOS shared mode this is the rate that
    /// must match CoreAudio's current nominal device rate; decoded track rates
    /// may differ and are resampled by Rodio.
    #[cfg(target_os = "macos")]
    fn output_sample_rate(&self) -> u32 {
        match self {
            StreamType::Rodio { sink, .. } => sink.config().sample_rate().get(),
        }
    }
}

pub(crate) fn apply_engine_volume(
    #[cfg_attr(not(target_os = "macos"), allow(unused_variables))] stream_opt: &Option<StreamType>,
    engine: &PlaybackEngine,
    volume: f32,
) {
    #[cfg(target_os = "macos")]
    if stream_opt
        .as_ref()
        .map(|stream| stream.set_coreaudio_hardware_volume(volume))
        .unwrap_or(false)
    {
        engine.set_volume(1.0);
        return;
    }

    engine.set_volume(volume);
}
