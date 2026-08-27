use super::*;

pub(crate) fn cpal_device_name(device: &rodio::cpal::Device) -> Option<String> {
    device
        .description()
        .ok()
        .map(|description| description.name().to_string())
}

/// Output stream type - either rodio or ALSA Direct.
pub(crate) enum StreamType {
    Rodio { sink: MixerDeviceSink },
    #[cfg(target_os = "linux")]
    AlsaDirect(Arc<qbz_audio::AlsaDirectStream>),
    /// Native JACK output (#263 Tier 3). QBZ as a JACK client with stable ports;
    /// NOT bit-perfect (resampled to the graph rate).
    #[cfg(target_os = "linux")]
    Jack(Arc<qbz_audio::JackStream>),
}

impl StreamType {
    pub(crate) fn rodio(sink: MixerDeviceSink) -> Self {
        StreamType::Rodio { sink }
    }

}

/// Volume is software-only: the CoreAudio hardware-volume path went with
/// macOS support, and ALSA/PipeWire/JACK never had an equivalent here.
/// `stream_opt` is kept in the signature so callers do not all change; it is
/// unused.
pub(crate) fn apply_engine_volume(
    _stream_opt: &Option<StreamType>,
    engine: &PlaybackEngine,
    volume: f32,
) {
    engine.set_volume(volume);
}
