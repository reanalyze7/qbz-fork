use super::QobuzClient;
use crate::error::{ApiError, Result};
use reqwest::StatusCode;

impl QobuzClient {
    // === 403 circuit breaker (issue #637) ===

    /// Short-circuit an authenticated request when the 403 breaker is open.
    /// Returns `Err(ForbiddenCircuitOpen)` — no network is touched — so a
    /// post-outage 403 storm cannot get the user's IP edge-blocked. Callers put
    /// this at the top of the hot streaming/favorites paths.
    pub(super) fn forbidden_guard(&self) -> Result<()> {
        // Test hook: `QBZ_FORCE_403=1` forces the breaker open so the whole 403
        // back-off path (no-network short-circuit + abort-fallback + the audible
        // "backing off" toast) can be smoke-tested on a HEALTHY account, which
        // otherwise can't reproduce the incident. Off by default; issue #637.
        if std::env::var_os("QBZ_FORCE_403").is_some() {
            return Err(ApiError::ForbiddenCircuitOpen(30));
        }
        if let Some(remaining) = self.forbidden_breaker.blocked_for() {
            return Err(ApiError::ForbiddenCircuitOpen(remaining.as_secs()));
        }
        Ok(())
    }

    /// Feed an authenticated response's status to the breaker: a 403 counts
    /// toward opening it; any success resets it. Other statuses are neutral
    /// (they have their own handling and must not open the breaker).
    pub(super) fn note_forbidden_status(&self, status: StatusCode) {
        if status == StatusCode::FORBIDDEN {
            if let Some(cooldown) = self.forbidden_breaker.record_forbidden() {
                log::warn!(
                    "[403-breaker] Repeated 403s from Qobuz — backing off for {}s (no network) \
                     to avoid an edge/IP block. See issue #637.",
                    cooldown.as_secs()
                );
            }
        } else if status.is_success() {
            self.forbidden_breaker.record_success();
        }
    }
}
