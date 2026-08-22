# crates/qbz-app/src/offline_mode/mod.rs (518 lines)

## Summary
The frontend-agnostic Offline Mode engine (ADR-006): tri-state
`OfflineMode` (Online/RealOffline/InducedOffline), `OfflineStatus`
broadcast struct, and `OfflineModeEngine` (holds the settings store,
induced/offline-session atomics, a `watch` channel, and the connectivity
snapshot; owns the process-wide `qbz_qobuz::offline_gate`). Already a
directory module with `connectivity.rs` and `store.rs` siblings — only
`mod.rs` itself is oversized, largely due to its ~200-line test module.

## Proposed split
- `offline_mode/mod.rs` (~25 lines) — module doc comment (D1-D3
  invariants), `pub mod connectivity; pub mod store; pub mod types; pub
  mod engine;` and the `pub use` re-exports.
- `offline_mode/types.rs` (~50 lines) — `OfflineMode` enum, `OfflineStatus`
  struct + its `is_offline`/`show_recovery_banner` methods,
  `default_status()` (lines 36-82).
- `offline_mode/engine.rs` (~230 lines) — `OfflineModeEngine` struct + all
  its methods (`new`, `init_for_user`, `teardown`, `subscribe`, `status`,
  `is_offline`, `settings`, `set_show_network_folders`, `set_induced`,
  `set_offline_session`, `on_connectivity`, `attach_connectivity`,
  `recompute`) + `Default` impl (lines 84-314).
- `offline_mode/tests.rs` (~200 lines) — the entire `#[cfg(test)] mod
  tests` block (lines 316-518), `use super::*;`.

## Re-export surface
`offline_mode/mod.rs` keeps `pub use types::{OfflineMode, OfflineStatus};
pub use engine::OfflineModeEngine;` alongside its existing `pub use
connectivity::{...}; pub use store::{...};` so
`qbz_app::offline_mode::{OfflineMode, OfflineStatus, OfflineModeEngine,
Connectivity, ConnectivityActor, ConnectivitySnapshot, OfflineModeSettings,
OfflineModeStore, QueuedScrobble}` all stay reachable unchanged — this
module is consumed by `qbz` (main app) and possibly `qbzd`, so the public
path must not move.

## Coupling / watch out
- `OfflineModeEngine::recompute()` is the single place that calls
  `qbz_qobuz::offline_gate::set_offline(...)` — this process-wide side
  effect must stay exactly where it is; do not duplicate or reorder it
  relative to the `status_tx.send_if_modified` broadcast.
- `set_induced()`'s issue #279 stream_first_track snapshot/restore logic
  is intricate (entering forces false + stashes prior value; exiting
  restores + clears the stash) — keep this whole block together in
  `engine.rs`, don't split further.
- Tests use a `static GATE_LOCK: Mutex<()>` to serialize access to the
  process-global `qbz_qobuz::offline_gate` — this must stay in
  `tests.rs` and every test must keep calling `serialize()` first, since
  the gate is shared across the whole test binary.
- `set_show_network_folders` is explicitly dead from the UI (per its
  2026-06-10 doc comment) but kept for Tauri-DB column compatibility —
  preserve that comment verbatim so a future cleanup pass doesn't delete
  it by mistake.

## Verify after split
- `cargo test -p qbz-app offline_mode` green (all 9 tests + the
  `#[tokio::test]`).
- `cargo build -p qbz-app` and whichever downstream crate (`qbz`, `qbzd`)
  uses `qbz_app::offline_mode::*`.
