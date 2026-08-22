# crates/qbz-media-controls/src/linux.rs (351 lines)

## Summary
Linux MPRIS backend built on `mpris-server` (chosen over souvlaki because it
exposes `RootInterface::desktop_entry()`, needed for GNOME's app-icon
resolution): runs the D-Bus server on a dedicated thread with its own
current-thread tokio runtime, receiving state updates over an async channel
from the app.

## Proposed split
By responsibility — a `linux/` module directory with a thin `mod.rs` holding
the shared types.

- `linux/mod.rs` (~55 lines) — consts (`BUS_SUFFIX`, `DESKTOP_ENTRY`,
  `IDENTITY`), `TRACK_SEQ` static, `State` struct, `Update` enum,
  `EventCb` type alias, `LinuxHandle` struct + its `MediaIntegration` impl,
  module declarations, and `pub use spawn::spawn;` re-export.
- `linux/metadata.rs` (~35 lines) — `map_status`, `build_metadata` (the
  `TrackMeta` → `mpris_server::Metadata` mapping, using `TRACK_SEQ` from
  `mod.rs`).
- `linux/root_iface.rs` (~55 lines) — the `QbzMpris` struct definition +
  `emit` helper + its `RootInterface` impl (the GNOME `desktop_entry` fix
  lives here).
- `linux/player_iface.rs` (~100 lines) — `QbzMpris`'s `PlayerInterface` impl
  (the transport controls: next/previous/pause/play/stop/seek/position/
  volume/loop/rate/shuffle/can-* getters).
- `linux/apply.rs` (~30 lines) — the `apply(server, state, update)` async fn
  that applies an `Update` to shared `State` and emits the matching
  `properties_changed`.
- `linux/spawn.rs` (~70 lines) — the public `spawn(on_event) -> Option<LinuxHandle>`
  entry point: thread spawn, tokio runtime build, `Server::new` registration,
  the `SleepInhibitor`-driven update loop.

## Re-export surface
`linux/mod.rs` is the public-API surface: `pub use spawn::spawn;` plus the
`LinuxHandle` struct defined directly in `mod.rs`, so
`crate::linux::spawn(...)` and `crate::linux::LinuxHandle` keep their current
paths for whatever calls into this backend (the media-controls facade that
picks Linux vs other platforms).

## Coupling / watch out
- `QbzMpris` (struct) is defined in `root_iface.rs` but also needs an
  `impl PlayerInterface for QbzMpris` block in `player_iface.rs` — Rust
  allows multiple `impl` blocks for one struct across files in the same
  module, so this is fine, but `player_iface.rs` must `use super::root_iface::QbzMpris;`
  (or re-export `QbzMpris` from `mod.rs` and have both impl files import
  from there instead, which is cleaner).
- `State` (shared mutable now-playing state) is read by both interface impls
  (`root_iface.rs`'s getters aren't state-backed, but `player_iface.rs`'s
  `playback_status`/`metadata`/`volume`/`position` all lock `self.state`) and
  written by `apply.rs` — keep `State`'s definition in `mod.rs` so all three
  files import the same type without ambiguity.
- `spawn.rs` constructs the initial `State` and `QbzMpris { on_event, state }`
  inline inside the spawned thread's `block_on` — after the split this needs
  `use super::{QbzMpris, State};` (or wherever they end up) plus
  `use super::apply::apply;`.
- The doc comment on the module (why `mpris-server` over souvlaki, the
  `DesktopEntry` GNOME-icon mechanism) is important context — keep it on
  `mod.rs` since that's the file other agents/readers will open first.
- `SleepInhibitor` (from `crate::inhibit`) is only used inside `spawn.rs`'s
  update loop (piggybacking on `Update::Playback` to hold/drop the idle
  inhibitor) — make sure that import and the inhibitor lifecycle stays in
  `spawn.rs`, not `apply.rs` (which only applies to the D-Bus `Server`/
  `State`, unaware of the inhibitor).

## Verify after split
- `cargo build -p qbz-media-controls`
- `cargo test -p qbz-media-controls` (no existing unit tests in this file;
  confirm the crate still builds cleanly for the Linux target).
- Grep for `linux::spawn` / `linux::LinuxHandle` usage across
  `qbz-media-controls` and its callers (likely `qbz-app` or `qbz`) to
  confirm the public path is unaffected.
- Manual smoke test on Linux (GNOME and/or KDE if available): media widget
  shows the correct app icon (the `desktop_entry` fix), play/pause/next/
  previous/seek/volume all round-trip correctly, and the sleep/idle
  inhibitor engages while playing and releases on pause/stop.
