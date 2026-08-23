/// Best-effort presence check against the TTL-cached device enumeration. Exact
/// device identity is refined in T10; here a substring match on either side
/// tolerates the CPAL-name vs `hw:` mismatch without false negatives on a match.
pub(super) fn device_is_present(state: &crate::api::ApiState, dev: &str) -> bool {
    cached_device_names(state)
        .iter()
        .any(|n| n == dev || n.contains(dev) || dev.contains(n.as_str()))
}

/// Device names, re-enumerated at most every 5 s (a `status` poll must not
/// re-scan CPAL on every call). On enumeration failure the timestamp is still
/// bumped so a broken audio stack is not hammered.
fn cached_device_names(state: &crate::api::ApiState) -> Vec<String> {
    use std::time::{Duration, Instant};
    let mut cache = match state.devices.lock() {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let fresh = cache
        .at
        .map(|t| t.elapsed() < Duration::from_secs(5))
        .unwrap_or(false);
    if !fresh {
        if let Ok(sinks) = qbz_audio::output_sinks::list_output_sinks() {
            cache.names = sinks.into_iter().map(|s| s.name).collect();
        }
        cache.at = Some(Instant::now());
    }
    cache.names.clone()
}
