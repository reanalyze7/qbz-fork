//! Pure frame accounting: timestamps in, one sample per window out.
//!
//! Split from the wiring so the arithmetic — which is the part that can be
//! wrong in a way nobody notices (an fps that flatters, a worst-frame that
//! resets at the wrong moment) — is unit-testable without a window, a GPU or
//! a running event loop. `Instant` is passed in, never read here.

use std::time::{Duration, Instant};

/// One window's worth of figures, emitted when the window closes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Sample {
    pub fps: f32,
    /// Mean interval between frames, milliseconds.
    pub frame_ms: f32,
    /// Longest single interval in the window, milliseconds.
    pub worst_ms: f32,
}

/// Counts frames over a fixed window and emits a `Sample` when it closes.
pub(crate) struct FrameMeter {
    window: Duration,
    started: Option<Instant>,
    last: Option<Instant>,
    frames: u32,
    worst: Duration,
}

impl FrameMeter {
    pub(crate) fn new(window: Duration) -> Self {
        Self { window, started: None, last: None, frames: 0, worst: Duration::ZERO }
    }

    /// Record one rendered frame. Returns a sample only on the tick that
    /// closes a window, and starts the next window from that same instant so
    /// no frame is counted twice or dropped between windows.
    pub(crate) fn record(&mut self, now: Instant) -> Option<Sample> {
        let started = *self.started.get_or_insert(now);
        if let Some(last) = self.last {
            // saturating: a clock that goes backwards must not panic here.
            self.worst = self.worst.max(now.saturating_duration_since(last));
        }
        self.last = Some(now);
        self.frames += 1;

        let elapsed = now.saturating_duration_since(started);
        if elapsed < self.window {
            return None;
        }
        let secs = elapsed.as_secs_f32();
        // The first frame of a window opens it rather than being an interval
        // inside it, so intervals = frames - 1. With a single frame there is
        // no interval to average and the mean is reported as 0.
        let intervals = self.frames.saturating_sub(1);
        let sample = Sample {
            fps: if secs > 0.0 { self.frames as f32 / secs } else { 0.0 },
            frame_ms: if intervals > 0 { secs * 1000.0 / intervals as f32 } else { 0.0 },
            worst_ms: self.worst.as_secs_f32() * 1000.0,
        };
        self.started = Some(now);
        self.frames = 0;
        self.worst = Duration::ZERO;
        Some(sample)
    }
}
