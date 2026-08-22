//! Forbidden (HTTP 403) circuit breaker.
//!
//! Issue #637: after a Qobuz outage + forced re-login, a user's account can be
//! transiently rejected (entitlement not yet restored, a wedged session, etc.).
//! Our prefetch scheduler re-drives `get_stream_url` / CMAF `session/start`
//! with no backoff, so a handful of legitimate 403s escalated into a sustained
//! ~2-3 req/s storm — which trips Qobuz's edge/WAF and turns the transient 403
//! into a persistent per-IP block (the "error decoding response body" lines in
//! the report are that HTML/empty WAF body hitting a `.json()` call).
//!
//! This breaker converts "hammer until IP-banned" into "back off and recover":
//! once a small number of 403s land in quick succession it opens for an
//! exponential cooldown, during which the hot streaming/favorites paths
//! short-circuit WITHOUT touching the network. After the cooldown a single
//! probe is allowed through; a success closes the breaker, another 403 re-opens
//! it immediately with a longer cooldown.
//!
//! Only genuine 403s feed it — 5xx/429/404/transport errors have their own
//! handling and must not open it.

use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Consecutive 403s (across the streaming/favorites paths) that open the breaker.
const OPEN_THRESHOLD: u32 = 3;
/// First cooldown once the breaker opens.
const BASE_COOLDOWN: Duration = Duration::from_secs(30);
/// Cooldown ceiling — doubling stops here.
const MAX_COOLDOWN: Duration = Duration::from_secs(120);

struct Inner {
    /// Consecutive 403s not yet cleared by a success. Not reset when the breaker
    /// opens, so the post-cooldown probe re-opens on a single further 403.
    consecutive: u32,
    /// When set and still in the future, the breaker is open until this instant.
    open_until: Option<Instant>,
    /// Cooldown applied on the next open (grows exponentially, capped).
    next_cooldown: Duration,
}

/// A shared, cheaply-clonable 403 circuit breaker. Uses a plain `Mutex` (no
/// `.await` held) so it is trivially callable from any async path.
pub struct ForbiddenBreaker {
    inner: Mutex<Inner>,
}

impl Default for ForbiddenBreaker {
    fn default() -> Self {
        Self::new()
    }
}

mod breaker;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
