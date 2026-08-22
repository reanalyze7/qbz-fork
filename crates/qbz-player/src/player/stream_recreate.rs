use super::*;

/// Inputs needed by both `Play` and `Stream` handlers to decide whether the
/// current output stream must be torn down and recreated.
pub(crate) struct StreamRecreateDecision {
    pub(crate) needs_new_stream: bool,
    pub(crate) format_changed: bool,
    pub(crate) dac_passthrough: bool,
    pub(crate) using_alsa_direct: bool,
    pub(crate) using_coreaudio_exclusive: bool,
    pub(crate) coreaudio_shared_rate_mismatch: Option<(u32, u32)>,
}

/// Read settings once and evaluate every condition that forces a stream
/// rebuild: any decoded-format change (sample rate / channels — the output
/// stream must follow the track's native rate on every backend, #449) and
/// CoreAudio shared-mode rate drift (CPAL caches the device rate at open
/// time, so when the OS nominal rate moves we must rebuild or playback runs
/// at the wrong speed).
///
/// `current_track_sample_rate` / `current_track_channels` describe the last
/// *decoded source* format and are compared to the incoming `sample_rate` /
/// `channels` — never to the output stream's hardware rate, which on macOS
/// shared mode may differ and is tracked separately via Rodio's sink config.
pub(crate) fn evaluate_stream_recreate(
    thread_settings: &Arc<Mutex<AudioSettings>>,
    stream_opt: &Option<StreamType>,
    current_track_sample_rate: Option<u32>,
    current_track_channels: Option<u16>,
    sample_rate: u32,
    channels: u16,
    context: &str,
) -> StreamRecreateDecision {
    let format_changed =
        current_track_sample_rate != Some(sample_rate) || current_track_channels != Some(channels);

    let settings_guard = thread_settings.lock().ok();

    let dac_passthrough = settings_guard
        .as_ref()
        .map(|s| cfg!(target_os = "linux") && s.dac_passthrough)
        .unwrap_or(false);

    let using_alsa_direct = settings_guard
        .as_ref()
        .and_then(|s| s.backend_type)
        .map(|b| b == AudioBackendType::Alsa)
        .unwrap_or(false);

    let using_coreaudio_exclusive = settings_guard
        .as_ref()
        .map(|s| {
            cfg!(target_os = "macos")
                && s.backend_type.unwrap_or(AudioBackendType::SystemDefault)
                    == AudioBackendType::SystemDefault
                && s.exclusive_mode
        })
        .unwrap_or(false);

    #[cfg(target_os = "macos")]
    let coreaudio_shared_rate_mismatch = settings_guard
        .as_ref()
        .and_then(|s| coreaudio_shared_rate_mismatch(s, stream_opt))
        .inspect(|(stream_rate, nominal_rate)| {
            log::warn!(
                "[CoreAudio] {} shared-mode output rate changed: stream {}Hz, device nominal {}Hz. Recreating stream to avoid wrong-speed playback.",
                context,
                stream_rate,
                nominal_rate
            );
        });
    #[cfg(not(target_os = "macos"))]
    let coreaudio_shared_rate_mismatch: Option<(u32, u32)> = {
        let _ = (stream_opt, context);
        None
    };

    drop(settings_guard);

    let needs_new_stream = compute_needs_new_stream(
        stream_opt.is_some(),
        format_changed,
        dac_passthrough,
        using_alsa_direct,
        using_coreaudio_exclusive,
        coreaudio_shared_rate_mismatch.is_some(),
    );

    StreamRecreateDecision {
        needs_new_stream,
        format_changed,
        dac_passthrough,
        using_alsa_direct,
        using_coreaudio_exclusive,
        coreaudio_shared_rate_mismatch,
    }
}

/// Pure decision rule for whether the output stream must be rebuilt.
///
/// Split out so the truth table can be unit-tested without faking a real
/// `MixerDeviceSink` or `AudioSettings` mutex.
fn compute_needs_new_stream(
    has_stream: bool,
    pub(crate) format_changed: bool,
    _dac_passthrough: bool,
    _using_alsa_direct: bool,
    _using_coreaudio_exclusive: bool,
    coreaudio_shared_rate_mismatch: bool,
) -> bool {
    // A decoded-format change (sample rate or channel count) requires a fresh
    // output stream on EVERY backend so the device follows the track's native
    // rate (#449). The bit-perfect flags used to gate this, which was correct
    // only while Stop dropped the stream; once that drop was deferred to avoid
    // a track-change click (e93fcaec), the default/PipeWire path stopped
    // switching rates and stayed locked to the first track. Same-rate tracks
    // keep reusing the stream (format_changed == false), preserving the click
    // fix.
    !has_stream || format_changed || coreaudio_shared_rate_mismatch
}

/// Try to create output stream using the backend system (if configured)
/// Returns None if backend system is not configured (backend_type = None)
///
/// For ALSA backend with hw: devices, may return AlsaDirect instead of Rodio stream.
