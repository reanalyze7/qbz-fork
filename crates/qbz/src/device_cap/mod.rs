//! Local output-device quality cap (#638 fix 3, spec Phase C).
//!
//! Caches the local DAC's detected rate ceiling, mapped to a Qobuz tier, so
//! the request-time resolution (`playback::local_playback_quality`) can clamp
//! the streaming-quality preference without probing per call. Detection is
//! the proven read-only `qbz_audio::query_dac_capabilities` (reads
//! `/proc/asound` and shells out to `pw-dump`; never opens a stream), so a
//! refresh runs inside `spawn_blocking` on EXPLICIT triggers only — startup,
//! the Settings toggle, an output-device/backend change, reset-to-defaults —
//! never on the playback hot path or the poll tick. A stale cap after a
//! hot-unplug (until the next device change), and an uncapped first track
//! when a session-restore play beats the startup refresh, are the accepted
//! trades — same class as the HiFi wizard's behavior; both self-heal.
//!
//! PRECEDENCE (owner decision, #638): the cap of the device ACTUALLY PLAYING
//! governs. This cache is for LOCAL playback only — the cast path must never
//! read it (the local DAC is not in a cast's signal path).

mod refresh;
mod state;
mod summary;
#[cfg(test)]
mod tests;

pub use refresh::refresh;
pub use state::cap;
pub use summary::summary;
