# crates/qbzd/src/mpris.rs (272 lines)

## Summary
MPRIS system-media-controls integration for qbzd: publishes daemon playback
over org.mpris.MediaPlayer2 via qbz-media-controls, with an outbound
CoreEvent-bus-to-OS-controls updater task and an inbound MediaEvent-to-core-
transport callback (media keys / desktop widget).

## Proposed split
Split along the file's own documented "two halves" (outbound / inbound),
plus enablement/spawn plumbing and the small state-mapping helpers:

- `mpris/mod.rs` (~60 lines) — module doc, imports, `MprisHandle` struct +
  `shutdown`, `enabled()`, re-exports.
- `mpris/spawn.rs` (~75 lines) — `spawn()`: builds the inbound callback,
  starts the outbound updater task (the async block that seeds state then
  loops on `bus.recv()`).
- `mpris/inbound.rs` (~65 lines) — `handle_media_event` + `spawn_advance`
  (media keys / desktop-widget commands -> core transport).
- `mpris/mapping.rs` (~30 lines) — `track_meta()`, `map_state()` (pure
  mapping helpers, easiest to unit test in isolation).
- `mpris/tests.rs` (~35 lines) — the existing `#[cfg(test)] mod tests`
  (map_state coverage + enabled() falsy-value classification), referencing
  `super::*`/`mapping::*` as needed.

## Re-export surface
`mpris/mod.rs` re-exports `MprisHandle` and the public `spawn()` function
(used by `daemon.rs` at boot) so `crate::mpris::spawn` / `crate::mpris::
MprisHandle` keep working unchanged.

## Coupling / watch out
- `spawn()` builds BOTH the inbound callback (via
  `qbz_media_controls::spawn(move |ev| ...)`) and the outbound updater in one
  function; splitting inbound handling into its own file means `spawn.rs`
  must import `inbound::handle_media_event` — keep the `Weak<AppRuntime>`
  upgrade-and-drop discipline documented in the file header (never let the
  split accidentally introduce a strong `Arc` that outlives the callback,
  which would break the #521 audio-release shutdown ordering).
- `enabled()` reads both an env var (`QBZD_MPRIS`) and `daemon_prefs` —
  keep it in `mod.rs` next to `MprisHandle` since `spawn()` calls it first
  thing.
- `spawn_advance` reaches into `qbz_app::playback_driver` and
  `daemon_prefs` — a cross-crate coupling point to double check compiles
  after the move (only its `use` lines change, not logic).

## Verify after split
- `cargo test -p qbzd mpris` — `map_state_covers_every_playback_state` and
  `enabled_defaults_on_and_respects_falsey_overrides` stay green.
- `cargo check -p qbzd`; smoke-test: run qbzd with a session bus present,
  confirm a KDE/GNOME media widget shows now-playing and media keys work
  (manual — no automated D-Bus test exists today).
