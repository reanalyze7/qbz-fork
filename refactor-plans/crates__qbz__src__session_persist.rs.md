# crates/qbz/src/session_persist.rs (326 lines)

## Summary
Session persistence (queue + playback state) glue over the frontend-
agnostic `SessionStore`: process-global store handle + cached gate flags,
capture-and-save, position-only quick save, and startup restore
(Phase A: paused restore + a pending-resume-position handoff for Phase B).

## Proposed split
By concern — lifecycle/gates vs track (de)serialization vs the two big
async operations:

- `session_persist/mod.rs` (~65 lines) — every `static` (`STORE`,
  `PERSIST_SESSION`, `RESUME_POSITION`, `PENDING_RESUME`,
  `PENDING_RESUME_TRACK`, `EXIT_CTX`), `Runtime` type alias,
  `bind_exit_ctx`, `save_on_exit`, `pub use` of submodules.
- `session_persist/lifecycle.rs` (~50 lines) — `init_for_user`,
  `set_gates`, `persist_enabled`, `take_resume_for`,
  `pending_resume_position`.
- `session_persist/convert.rs` (~75 lines) — `repeat_to_str`,
  `repeat_from_str`, `to_persisted`, `from_persisted` (the pure
  `QueueTrack` <-> `PersistedQueueTrack` mapping).
- `session_persist/save.rs` (~65 lines) — `capture_and_save`,
  `save_position`.
- `session_persist/restore.rs` (~65 lines) — `restore`.

## Re-export surface
`session_persist/mod.rs` stays the `mod session_persist;` target with every
static defined there. Public fns called from `main.rs`/playback code
(`bind_exit_ctx`, `save_on_exit`, `init_for_user`, `set_gates`,
`persist_enabled`, `take_resume_for`, `pending_resume_position`,
`capture_and_save`, `save_position`, `restore`) re-exported via `pub use
lifecycle::*; pub use save::*; pub use restore::restore;` so
`crate::session_persist::X` is unchanged.

## Coupling / watch out
- Every static (`STORE`, `PERSIST_SESSION`, `RESUME_POSITION`,
  `PENDING_RESUME`, `PENDING_RESUME_TRACK`) is touched from at least two of
  the four submodules (e.g. `PENDING_RESUME`/`PENDING_RESUME_TRACK` are
  written by `restore.rs` and read/cleared by `lifecycle.rs`'s
  `take_resume_for`) — keep them all in `mod.rs`, every submodule does
  `use super::{STORE, PERSIST_SESSION, ...};`.
- `save_on_exit` (mod.rs) calls `persist_enabled()` (lifecycle.rs) and
  `capture_and_save` (save.rs) — cross-file calls via `use
  lifecycle::persist_enabled; use save::capture_and_save;` (or full paths).
- `capture_and_save`'s crash-chain-level>=3 special case ("keeping the
  preserved snapshot on disk") is a real bugfix with specific reasoning in
  the comment — preserve verbatim, do not treat as dead code.
- The resume-position handoff is a two-phase protocol
  (`PENDING_RESUME`/`PENDING_RESUME_TRACK` set by `restore`, consumed once
  by `take_resume_for`, peeked without consuming by
  `pending_resume_position`) — all three must stay logically coupled even
  though they're split across `restore.rs`/`lifecycle.rs`; the module doc
  comment explains the two-phase design (Phase A paused restore, Phase B
  consumes on first play) — keep that comment attached to whichever file
  ends up as `mod.rs`.
- `from_persisted`'s doc comments explain two DELIBERATE data-loss points
  (no `version`, no `album_version`, no `context_kind`/`context_id` in the
  persisted schema) — these are pre-existing, documented limitations, not
  bugs to fix during the split.

## Verify after split
- `cargo build -p qbz` (no `#[cfg(test)]` in this file — flag as a gap;
  `repeat_to_str`/`repeat_from_str`/`to_persisted`/`from_persisted` are
  easy, valuable unit-test candidates for a real split PR).
- Manual test: enable persist_session + resume_playback_position, play a
  track partway, quit, relaunch — confirm the queue restores paused and the
  saved position applies on first play of the restored track (and NOT on
  playing a different track first).
