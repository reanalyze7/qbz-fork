// crates/qbzd/src/state/ — shared in-memory daemon state (one
// `Arc<Mutex<DaemonShared>>` shared by the playback driver + the HTTP API).
// Fields land now; real sources wire in as each producing task lands
// (T3 driver/audio, T6 HTTP server, T7 transport, T9/T10 QConnect).
#[cfg(test)]
mod tests;

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct LatchedErrors {
    // 01 §9.4 — drain-once channels become latches
    pub stream: Option<String>,
    pub auth: Option<String>,
    pub transport: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthState {
    NeedsAuth,
    Restoring,
    LoggedIn,
} // 01 §6.2 machine

pub struct DaemonShared {
    // one Arc<Mutex<...>> shared by driver + API
    pub auth: AuthState,
    pub user_id: Option<u64>,
    pub subscription: Option<String>,
    pub last_errors: LatchedErrors,
    pub driver_last_tick: Option<std::time::Instant>,
    pub muted: bool,
    pub premute_volume: f32,
    pub started_at: std::time::Instant,
    pub startup_warnings: u32,
    /// T11 (`POST /api/settings/reload`, 02 §3.3.17): a fingerprint of the
    /// credential-file token currently applied to the live session, so reload
    /// can tell "new token on disk" (re-login) from "same token, unrelated
    /// nudge" (no-op) without keeping a second copy of the secret in memory.
    /// `None` whenever the daemon is not LoggedIn against a known token
    /// (cleared alongside every `set_needs_auth`).
    pub credential_fingerprint: Option<u64>,
    /// Coarse network-reachability signal for `/api/status`'s `network.online`
    /// (01 §9.3). Latched ONLY from real network-class outcomes — never active
    /// probing: false on an auth-retry/credential-reload network-class failure
    /// or a QConnect reconnect-exhausted, true on a successful login/restore
    /// (`restore_activate`, which reload's own credential validation also
    /// funnels through) or a successful QConnect (re)connect. Defaults true
    /// (optimistic) until the first outcome latches it.
    pub network_online: std::sync::atomic::AtomicBool,
}

impl DaemonShared {
    /// Read the latched network-reachability signal (§9.3). `Relaxed` is
    /// sufficient — this is a coarse status flag read under the same
    /// `Mutex<DaemonShared>` guard as every other field here, not a
    /// synchronization primitive of its own.
    pub fn network_online(&self) -> bool {
        self.network_online.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Latch the network-reachability signal. See the field doc for exactly
    /// which call sites are allowed to call this.
    pub fn set_network_online(&self, online: bool) {
        self.network_online
            .store(online, std::sync::atomic::Ordering::Relaxed);
    }
}

/// A non-reversible-in-practice fingerprint of a credential token (SipHash via
/// the stdlib default hasher) — used ONLY to detect "the file changed", never
/// to reconstruct the token. Keeps `DaemonShared` from holding a second live
/// copy of the secret alongside the credential file + the Qobuz client.
pub fn token_fingerprint(token: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    token.hash(&mut hasher);
    hasher.finish()
}
