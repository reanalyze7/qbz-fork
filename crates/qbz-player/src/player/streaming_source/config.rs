//! Streaming buffer sizing: [`StreamingConfig`]. The process-wide initial
//! buffer cap (issue #331) lives in `cap.rs`.

use super::cap::{raw_initial_buffer_for_speed, MAX_INITIAL_BUFFER_BYTES};

/// Configuration for the streaming buffer
#[derive(Debug, Clone)]
pub struct StreamingConfig {
    /// Minimum bytes to buffer before allowing reads (for format detection)
    pub initial_buffer_bytes: usize,
    /// Maximum buffer size before backpressure (not enforced, just for info)
    pub max_buffer_bytes: usize,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            // 512KB default - enough for format headers and ~2-5 seconds of audio
            // This allows playback to start quickly while still having enough
            // buffer to handle network jitter
            initial_buffer_bytes: 512 * 1024,
            // 100MB max buffer
            max_buffer_bytes: 100 * 1024 * 1024,
        }
    }
}

impl StreamingConfig {
    /// Create config from buffer seconds and approximate bitrate
    ///
    /// For Hi-Res FLAC at 192kHz/24bit stereo, bitrate is roughly 9.2 Mbps
    /// We estimate ~1MB per second as a conservative approximation
    pub fn from_seconds(seconds: u8) -> Self {
        // Minimum 256KB to ensure format detection works
        let bytes = ((seconds as usize) * 1024 * 1024).max(256 * 1024);
        Self {
            initial_buffer_bytes: bytes,
            max_buffer_bytes: 100 * 1024 * 1024,
        }
    }

    /// Create a minimal config for fastest startup
    /// Uses smallest buffer that still allows format detection (~256KB)
    pub fn fast_start() -> Self {
        Self {
            initial_buffer_bytes: 256 * 1024,
            max_buffer_bytes: 100 * 1024 * 1024,
        }
    }

    /// Create config dynamically based on measured download speed
    ///
    /// - Very fast (>10 MB/s): 256KB (instant start)
    /// - Fast (5-10 MB/s): 384KB
    /// - Normal (2-5 MB/s): 512KB
    /// - Slow (1-2 MB/s): 1MB (more buffer to prevent stutter)
    /// - Very slow (<1 MB/s): 2MB
    ///
    /// Result is clamped to the process-wide cap configured via
    /// [`set_max_initial_buffer_bytes`] (typically derived from the host's
    /// memory profile — see qbz-core's system_capabilities). On
    /// memory-constrained hosts the slow-connection branches would
    /// otherwise inflate to 2 MB, which is exactly the wrong direction
    /// when "slow connection" is itself a symptom of swap thrash
    /// (issue #331, Pi 3B).
    pub fn from_speed_mbps(speed_mbps: f64) -> Self {
        let cap = MAX_INITIAL_BUFFER_BYTES.load(std::sync::atomic::Ordering::Relaxed);
        let cfg = Self::from_speed_mbps_with_cap(speed_mbps, cap);

        if cfg.initial_buffer_bytes < raw_initial_buffer_for_speed(speed_mbps) {
            log::info!(
                "Dynamic buffer: {:.1} MB/s detected → {}KB (capped from {}KB by host memory profile)",
                speed_mbps,
                cfg.initial_buffer_bytes / 1024,
                raw_initial_buffer_for_speed(speed_mbps) / 1024
            );
        } else {
            log::info!(
                "Dynamic buffer: {:.1} MB/s detected → {}KB initial buffer",
                speed_mbps,
                cfg.initial_buffer_bytes / 1024
            );
        }

        cfg
    }

    /// Pure variant of [`from_speed_mbps`] — derives the speed-based
    /// initial buffer and clamps to `cap` without touching global state
    /// or logging. Exposed for unit tests; production callers should use
    /// `from_speed_mbps`, which reads the process-wide cap.
    pub fn from_speed_mbps_with_cap(speed_mbps: f64, cap: usize) -> Self {
        let raw_initial_buffer = raw_initial_buffer_for_speed(speed_mbps);
        Self {
            initial_buffer_bytes: raw_initial_buffer.min(cap),
            max_buffer_bytes: 100 * 1024 * 1024,
        }
    }
}
