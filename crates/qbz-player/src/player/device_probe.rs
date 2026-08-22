use super::*;

#[cfg(target_os = "macos")]
fn uses_coreaudio_system_default(settings: &AudioSettings) -> bool {
    settings
        .backend_type
        .unwrap_or(AudioBackendType::SystemDefault)
        == AudioBackendType::SystemDefault
}

/// `evaluate_stream_recreate` runs every track change on the audio thread,
/// and `coreaudio_nominal_rate` issues two CoreAudio HAL queries per call
/// (`resolve_output_device_id` + `get_nominal_sample_rate`). A short-lived
/// cache absorbs the bulk of those repeats — long enough to skip duplicate
/// queries within an album, short enough to notice when the user changes the
/// system audio rate.
#[cfg(target_os = "macos")]
const COREAUDIO_RATE_CACHE_TTL: std::time::Duration = std::time::Duration::from_millis(750);

#[cfg(target_os = "macos")]
struct CachedNominalRate {
    cached_at: std::time::Instant,
    device_name: Option<String>,
    rate: Option<u32>,
}

#[cfg(target_os = "macos")]
std::thread_local! {
    static COREAUDIO_NOMINAL_RATE_CACHE: std::cell::RefCell<Option<CachedNominalRate>> =
        const { std::cell::RefCell::new(None) };
}

#[cfg(target_os = "macos")]
fn query_macos_nominal_rate(device_name: Option<&str>) -> Option<u32> {
    let device_id = qbz_audio::coreaudio_direct::resolve_output_device_id(device_name)
        .inspect_err(|e| {
            log::warn!(
                "[CoreAudio] Failed to resolve output device for rate check: {}",
                e
            )
        })
        .ok()?;

    qbz_audio::coreaudio_direct::get_nominal_sample_rate(device_id)
        .inspect_err(|e| log::warn!("[CoreAudio] Failed to query nominal rate: {}", e))
        .ok()
}

#[cfg(target_os = "macos")]
pub(crate) fn coreaudio_nominal_rate(settings: &AudioSettings) -> Option<u32> {
    if !uses_coreaudio_system_default(settings) {
        return None;
    }

    let device_name = settings.output_device.clone();
    let now = std::time::Instant::now();

    COREAUDIO_NOMINAL_RATE_CACHE.with(|cell| {
        let mut cell = cell.borrow_mut();
        if let Some(cached) = cell.as_ref() {
            if cached.device_name == device_name
                && now.duration_since(cached.cached_at) < COREAUDIO_RATE_CACHE_TTL
            {
                return cached.rate;
            }
        }
        let rate = query_macos_nominal_rate(device_name.as_deref());
        *cell = Some(CachedNominalRate {
            cached_at: now,
            device_name,
            rate,
        });
        rate
    })
}

#[cfg(target_os = "macos")]
pub(crate) fn coreaudio_shared_rate_mismatch(
    settings: &AudioSettings,
    stream_opt: &Option<StreamType>,
) -> Option<(u32, u32)> {
    if settings.exclusive_mode || !uses_coreaudio_system_default(settings) {
        return None;
    }

    let stream_rate = stream_opt.as_ref().map(StreamType::output_sample_rate)?;
    let nominal_rate = coreaudio_nominal_rate(settings)?;

    (stream_rate != nominal_rate).then_some((stream_rate, nominal_rate))
}
