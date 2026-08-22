use super::{ForbiddenBreaker, Inner, BASE_COOLDOWN, MAX_COOLDOWN, OPEN_THRESHOLD};
use std::sync::Mutex;
use std::time::{Duration, Instant};

impl ForbiddenBreaker {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(Inner {
                consecutive: 0,
                open_until: None,
                next_cooldown: BASE_COOLDOWN,
            }),
        }
    }

    /// If the breaker is open, returns the remaining cooldown so the caller can
    /// short-circuit (and log how long it is backing off). Returns `None` when
    /// closed OR when the cooldown has just elapsed — in the latter case the
    /// open state is cleared so exactly the next call is a half-open probe.
    pub fn blocked_for(&self) -> Option<Duration> {
        let mut g = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(until) = g.open_until {
            let now = Instant::now();
            if now < until {
                return Some(until - now);
            }
            // Cooldown elapsed: half-open. Let the next call probe the network.
            g.open_until = None;
        }
        None
    }

    /// Record a successful authenticated response: fully resets the breaker.
    pub fn record_success(&self) {
        let mut g = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        g.consecutive = 0;
        g.open_until = None;
        g.next_cooldown = BASE_COOLDOWN;
    }

    /// Record a 403. Opens (or re-opens) the breaker once the threshold is hit.
    /// Returns the cooldown it opened with, if it opened on this call — for
    /// logging.
    pub fn record_forbidden(&self) -> Option<Duration> {
        let mut g = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        g.consecutive = g.consecutive.saturating_add(1);
        if g.consecutive >= OPEN_THRESHOLD {
            let cooldown = g.next_cooldown;
            g.open_until = Some(Instant::now() + cooldown);
            g.next_cooldown = (g.next_cooldown * 2).min(MAX_COOLDOWN);
            Some(cooldown)
        } else {
            None
        }
    }
}
