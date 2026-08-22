//! Process-wide cap on the dynamically-derived initial buffer size
//! (issue #331): a single `static` so it can be set once at startup from
//! the host's detected memory profile and read everywhere else.

/// Speed-driven initial buffer size, before any cap is applied.
/// Pure function — used by both `from_speed_mbps` and
/// `from_speed_mbps_with_cap` so they share the same ladder.
pub(super) fn raw_initial_buffer_for_speed(speed_mbps: f64) -> usize {
    if speed_mbps >= 10.0 {
        256 * 1024 // 256KB - instant start for very fast connections
    } else if speed_mbps >= 5.0 {
        384 * 1024 // 384KB
    } else if speed_mbps >= 2.0 {
        512 * 1024 // 512KB - default
    } else if speed_mbps >= 1.0 {
        1024 * 1024 // 1MB - more buffer for slower connections
    } else {
        2 * 1024 * 1024 // 2MB - maximum buffer for very slow connections
    }
}

/// Process-wide cap for dynamically-derived initial buffer sizes.
/// Defaults to `usize::MAX` (no cap) so behavior is unchanged unless the
/// host explicitly configures it via [`set_max_initial_buffer_bytes`] —
/// typically once at process start, derived from the detected memory
/// profile.
pub(super) static MAX_INITIAL_BUFFER_BYTES: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(usize::MAX);

/// Set the process-wide cap for `StreamingConfig::from_speed_mbps`.
/// Subsequent calls to that constructor clamp their result to this cap.
pub fn set_max_initial_buffer_bytes(bytes: usize) {
    MAX_INITIAL_BUFFER_BYTES.store(bytes, std::sync::atomic::Ordering::Relaxed);
}

/// Read the current cap. Mainly useful for tests.
pub fn max_initial_buffer_bytes() -> usize {
    MAX_INITIAL_BUFFER_BYTES.load(std::sync::atomic::Ordering::Relaxed)
}
