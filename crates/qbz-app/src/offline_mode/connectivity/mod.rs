//! Connectivity actor — the robust replacement for Tauri's poll-only checker.
//!
//! Tauri's checker (issue #467 era) raced three hostname-based generate_204
//! probes from a frontend `setInterval`, with a process-global 2-consecutive-
//! failure counter. Its residual false-offline vectors (review report 02 §4):
//! every probe needs DNS, two of three endpoints are Google infra (Pi-hole),
//! unspaced counting defeats the hysteresis, nothing resets on suspend/resume,
//! and captive-portal redirects count as ONLINE.
//!
//! This actor is Rust-owned and layered (spec §3.2):
//!
//! 1. **OS route signal (Linux)** — no IPv4/IPv6 default route ⇒ `Down`
//!    immediately, no probe needed (pulling the cable / wifi off detects in
//!    one tick instead of two failed 8s probes). Sandbox-safe `/proc` reads.
//! 2. **Passive liveness** — audio bytes flowing within the last 45 s ⇒ `Up`
//!    by definition (`qbz_audio::network_throttle`), no probe traffic while
//!    streaming. Same rule as #467's Fix E.
//! 3. **Hardened probe set** — one IP-LITERAL probe (DNS-independent — the
//!    DNS-hiccup false-offline vector dies here), vendor diversity (Cloudflare
//!    / Google / Microsoft), strict response validation, and redirects count
//!    as CAPTIVE PORTAL, never as success.
//! 4. **Asymmetric hysteresis** — one confirmed success flips `Up` instantly;
//!    flipping `Down` from `Up` requires a fresh CONFIRMATION BURST (immediate
//!    short re-probes) so a single lost race never declares offline, and the
//!    confirmation is time-bounded (stale failures don't count).
//! 5. **Suspend/resume guard** — a wall-clock jump without matching monotonic
//!    progress discards accumulated failures before judging.
//!
//! State broadcasts over a `tokio::sync::watch` channel; the offline-mode
//! engine subscribes and derives the app mode from it.

mod actor;
mod judge;
mod loop_body;
mod probe;
mod route;
#[cfg(test)]
mod tests;
mod types;

pub use actor::ConnectivityActor;
pub use judge::ConnectivityJudge;
pub use probe::probe_all;
pub use route::has_default_route;
pub use types::{Connectivity, ConnectivitySnapshot, JudgeAction, ProbeOutcome};
