use std::time::{Duration, Instant};

use super::types::{Connectivity, ConnectivitySnapshot, JudgeAction, ProbeOutcome, CONFIRM_DELAYS};

/// Pure decision core — injected outcomes, no sockets. Unit-testable.
#[derive(Debug)]
pub struct ConnectivityJudge {
    snapshot: ConnectivitySnapshot,
    /// Pending down-confirmation: how many burst steps are left.
    confirm_steps_left: usize,
    /// When the failing streak started (time-bounds the confirmation).
    first_failure_at: Option<Instant>,
}

impl ConnectivityJudge {
    pub fn new() -> Self {
        Self {
            snapshot: ConnectivitySnapshot::default(),
            confirm_steps_left: 0,
            first_failure_at: None,
        }
    }

    pub fn snapshot(&self) -> ConnectivitySnapshot {
        self.snapshot
    }

    /// A definitive OS-level signal: no default route at all.
    pub fn on_no_route(&mut self) {
        self.confirm_steps_left = 0;
        self.first_failure_at = None;
        self.snapshot = ConnectivitySnapshot {
            state: Connectivity::Down,
            captive_portal: false,
        };
    }

    /// Audio bytes (or other Qobuz traffic) observed recently.
    pub fn on_liveness(&mut self) {
        self.confirm_steps_left = 0;
        self.first_failure_at = None;
        self.snapshot = ConnectivitySnapshot {
            state: Connectivity::Up,
            captive_portal: false,
        };
    }

    /// Discard any failing streak (suspend/resume, manual mode change).
    pub fn reset_streak(&mut self) {
        self.confirm_steps_left = 0;
        self.first_failure_at = None;
    }

    pub fn on_probe(&mut self, outcome: ProbeOutcome, now: Instant) -> JudgeAction {
        match outcome {
            ProbeOutcome::Success => {
                // Asymmetric: one confirmed success flips Up instantly.
                self.confirm_steps_left = 0;
                self.first_failure_at = None;
                self.snapshot = ConnectivitySnapshot {
                    state: Connectivity::Up,
                    captive_portal: false,
                };
                JudgeAction::Idle
            }
            ProbeOutcome::Failure | ProbeOutcome::CaptivePortal => {
                let captive = outcome == ProbeOutcome::CaptivePortal;
                match self.snapshot.state {
                    Connectivity::Up => {
                        // Was up: never flip on one loss — start/advance the
                        // confirmation burst.
                        match self.first_failure_at {
                            None => {
                                self.first_failure_at = Some(now);
                                self.confirm_steps_left = CONFIRM_DELAYS.len();
                            }
                            Some(start) => {
                                // Time-bound: a stale streak (e.g. ticks that
                                // straddled a suspend) restarts instead of
                                // accumulating.
                                if now.duration_since(start) > Duration::from_secs(120) {
                                    self.first_failure_at = Some(now);
                                    self.confirm_steps_left = CONFIRM_DELAYS.len();
                                }
                            }
                        }
                        if self.confirm_steps_left > 0 {
                            let idx = CONFIRM_DELAYS.len() - self.confirm_steps_left;
                            self.confirm_steps_left -= 1;
                            JudgeAction::ConfirmAfter(CONFIRM_DELAYS[idx])
                        } else {
                            // Burst exhausted and still failing: confirmed down.
                            self.snapshot = ConnectivitySnapshot {
                                state: Connectivity::Down,
                                captive_portal: captive,
                            };
                            self.first_failure_at = None;
                            JudgeAction::Idle
                        }
                    }
                    Connectivity::Down | Connectivity::Unknown => {
                        // Already down (or first-ever verdict): no burst needed.
                        self.snapshot = ConnectivitySnapshot {
                            state: Connectivity::Down,
                            captive_portal: captive,
                        };
                        JudgeAction::Idle
                    }
                }
            }
        }
    }
}

impl Default for ConnectivityJudge {
    fn default() -> Self {
        Self::new()
    }
}
