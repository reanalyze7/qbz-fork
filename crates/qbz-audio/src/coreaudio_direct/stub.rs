// ---- Non-macOS stubs ----

/// Query supported sample rates (stub for non-macOS)
pub fn query_supported_sample_rates(_device_name: &str) -> Result<Vec<u32>, String> {
    Ok(Vec::new())
}

/// Get the current nominal sample rate (stub for non-macOS)
pub fn get_nominal_sample_rate_by_name(_device_name: &str) -> Result<u32, String> {
    Err("CoreAudio is only available on macOS".to_string())
}

/// Set the nominal sample rate (stub for non-macOS)
pub fn set_nominal_sample_rate_by_name(
    _device_name: &str,
    _target_rate: u32,
) -> Result<(), String> {
    Err("CoreAudio is only available on macOS".to_string())
}

/// Non-macOS placeholder so shared backend/player signatures can mention the type.
#[derive(Debug)]
pub struct CoreAudioExclusiveGuard;
