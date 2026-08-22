use std::sync::OnceLock;
use std::sync::RwLock;
use std::time::Instant;

use super::{BANDWIDTH_EMA_ALPHA, CAUTIOUS_RATIO, PANIC_WINDOW_SECS, RELAXED_RATIO, SURVIVING_RATIO};

#[derive(Debug, Default)]
pub(super) struct ThrottleInner {
    /// Exponential moving average of observed segment-download bandwidth
    /// in MB/s. `None` until the first sample arrives.
    bandwidth_ema_mbps: Option<f64>,
    /// Timestamp of the most recent audio underrun. `None` means we have
    /// not observed one this session.
    last_underrun: Option<Instant>,
    /// Timestamp of the most recent successful segment download. `None`
    /// until the first segment lands. Used as a positive liveness signal
    /// for the offline detector (issue #467).
    last_successful_download: Option<Instant>,
}

pub struct ThrottleState {
    pub(super) inner: RwLock<ThrottleInner>,
}

static GLOBAL: OnceLock<ThrottleState> = OnceLock::new();

/// Singleton accessor. Lazily initialized on first call.
pub fn state() -> &'static ThrottleState {
    GLOBAL.get_or_init(|| ThrottleState {
        inner: RwLock::new(ThrottleInner::default()),
    })
}

impl ThrottleState {
    /// Feed a fresh per-segment bandwidth measurement (MB/s). Called from
    /// the CMAF streaming loop every few segments.
    pub fn record_segment_bandwidth(&self, mbps: f64) {
        if let Ok(mut inner) = self.inner.write() {
            // Liveness signal (issue #467): reaching this path means a segment
            // batch just downloaded — bytes are flowing, so we are online,
            // independent of whether the throughput sample is usable for the
            // EMA. Recorded before the validity guard on purpose.
            inner.last_successful_download = Some(Instant::now());
            if mbps.is_finite() && mbps > 0.0 {
                inner.bandwidth_ema_mbps = Some(match inner.bandwidth_ema_mbps {
                    Some(prev) => prev * (1.0 - BANDWIDTH_EMA_ALPHA) + mbps * BANDWIDTH_EMA_ALPHA,
                    None => mbps,
                });
            }
        }
    }

    /// Signal that an audio buffer underrun just happened. Forces the
    /// throttle into panic mode for `PANIC_WINDOW_SECS`.
    pub fn record_underrun(&self) {
        if let Ok(mut inner) = self.inner.write() {
            inner.last_underrun = Some(Instant::now());
        }
    }

    /// Current EMA bandwidth in MB/s, or `None` if no samples yet.
    pub fn current_bandwidth_mbps(&self) -> Option<f64> {
        self.inner.read().ok().and_then(|i| i.bandwidth_ema_mbps)
    }

    /// Seconds since the last successful segment download, or `None` if no
    /// bytes have been pulled this session. Positive liveness signal for the
    /// offline detector (issue #467): if segments are flowing we are online
    /// by definition, regardless of what a connectivity probe reports.
    pub fn seconds_since_download(&self) -> Option<u64> {
        self.inner
            .read()
            .ok()
            .and_then(|i| i.last_successful_download)
            .map(|t| t.elapsed().as_secs())
    }

    /// True when an underrun was recorded within `PANIC_WINDOW_SECS`.
    pub fn in_panic_mode(&self) -> bool {
        let inner = match self.inner.read() {
            Ok(g) => g,
            Err(_) => return false,
        };
        match inner.last_underrun {
            Some(t) => t.elapsed().as_secs() < PANIC_WINDOW_SECS,
            None => false,
        }
    }

    /// Decide the prefetch cap given the current track's playback rate and
    /// the memory-profile default. The cap is always clamped to `[0, default_cap]`
    /// — we never *raise* prefetch above the memory profile, only restrict.
    pub fn current_prefetch_cap(&self, playback_mbps: f64, default_cap: usize) -> usize {
        if self.in_panic_mode() {
            return 0;
        }
        let bw = match self.current_bandwidth_mbps() {
            Some(v) => v,
            // No samples yet — trust the memory profile default. The first
            // segment of the first track will land within a few seconds and
            // give us real numbers.
            None => return default_cap,
        };
        let ratio = if playback_mbps > 0.0 {
            bw / playback_mbps
        } else {
            f64::INFINITY
        };
        if ratio <= SURVIVING_RATIO {
            0
        } else if ratio <= CAUTIOUS_RATIO {
            1.min(default_cap)
        } else if ratio <= RELAXED_RATIO {
            2.min(default_cap)
        } else {
            default_cap
        }
    }
}
