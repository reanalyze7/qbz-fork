// crates/qbzd/src/api/playback/ — routes 4-12 (02-cli-and-api.md §3.3.4-12):
// GET /api/now-playing + the 8 POST /api/playback/* transport routes.
//
// 409 needs_auth (01-architecture.md §6.2) is gated per-route by reading the
// per-route Errors column in 02 §3.3, not blanket-applied: `/api/now-playing`
// gates unconditionally (§3.3.4); `play`/`toggle` gate ONLY the cold-start
// branch (§3.3.5-8 "cold-start needs a session"); `next`/`previous` gate
// unconditionally before running the advance ritual (§3.3.9-10); `pause`/
// `stop`/`seek`/`volume` never cold-start and are NOT listed with needs_auth
// in their own Errors columns (§3.3.11-12), so they act on whatever is
// already loaded regardless of auth state.
//
// DSD-direct guard: `Player::is_dsd_direct_active()` (qbz-player/src/player/
// mod.rs:4893, "True while a DoP stream is active (volume fixed, seek
// unsupported)") is the player's own guard — previously unconsumed anywhere
// in the workspace. `seek`/`volume` (incl. the `mute` body form) check it
// FIRST and refuse 409 rather than silently no-op (the brief's explicit
// requirement — a silent no-op reads as broken, 02 §1.4).
//
// Mute is daemon-owned state in `DaemonShared.{muted, premute_volume}` (T2
// seam), NOT the desktop's process statics (`crates/qbz/src/playback.rs:
// 3907-3930`) — same semantics (stash-then-zero / restore), different owner.
// The reported `playback.volume` is always the NOMINAL (pre-mute) level, both
// muted and unmuted: `premute_volume` when muted, the live player volume
// otherwise. This is what makes `now`'s and `mute`'s human lines ("vol 80%",
// "muted (was 80%)") trivial reads of one JSON field, and mirrors the
// desktop's PREMUTE_VOLUME/MUTED pair exactly, just relocated.
mod cold_start;
mod errors;
mod now_playing;
mod queue_modes;
mod seek;
#[cfg(test)]
mod tests;
mod transport;
mod volume;

pub use now_playing::now_playing;
pub use queue_modes::{repeat, shuffle};
pub use seek::seek;
pub use transport::{next, pause, play, previous, stop, toggle};
pub use volume::volume;

/// Streaming quality for a play-time resolve, from the daemon's persisted
/// prefs — the SAME key contract `daemon.rs` uses to seed the driver's
/// `DriverDeps.quality` closure at boot (01 §10.3), so a cold-start play and
/// the next auto-advance never pick different tiers.
pub(crate) fn resolve_quality(state: &super::ApiState) -> qbz_models::Quality {
    let prefs = qbz_app::settings::daemon_prefs::load_at(&state.roots.data);
    qbz_app::playback_driver::quality_from_key(&prefs.streaming_quality)
}
