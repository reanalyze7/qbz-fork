# crates/qbz/src/offline_mode.rs (330 lines)

## Summary
Slint-side glue for the shared offline-mode engine (ADR-006): process
globals (engine, connectivity actor), per-user subscription-state binding
(D4 grace/purge), and UI forwarding of engine status into `OfflineState`.

## Proposed split
By concern — engine/connectivity lifecycle vs subscription-state (D4) vs
UI forwarding, these are three fairly independent subsystems glued by
shared statics:

- `offline_mode/mod.rs` (~70 lines) — the four `static` globals (`ENGINE`,
  `CONNECTIVITY`, `SUBSCRIPTION`), `engine()`, `start()`, `request_recheck()`,
  `check_now()`, `pub use` of submodules.
- `offline_mode/ui_forward.rs` (~65 lines) — `seed_settings`,
  `start_ui_forwarder`, `apply_status` (the engine -> `OfflineState`/
  `SettingsState` Slint global mirroring).
- `offline_mode/subscription.rs` (~130 lines) — `user_data_dir`,
  `init_for_user`, `teardown`, `now_unix_secs`, `subscription_mark_valid`,
  `subscription_mark_invalid`, `offline_playback_allowed` — still at budget,
  could split `mark_valid`/`mark_invalid` (~50 lines) from
  `offline_playback_allowed`/`user_data_dir`/`now_unix_secs` (~40 lines) if
  it creeps further.
- `offline_mode/purge.rs` (~50 lines) — `spawn_subscription_purge_check`
  (the D4 activation-time cache purge consumer).

## Re-export surface
`offline_mode/mod.rs` stays the `mod offline_mode;` target. All `pub fn`s
(`engine`, `start`, `request_recheck`, `check_now`, `seed_settings`,
`start_ui_forwarder`, `user_data_dir`, `init_for_user`, `teardown`,
`subscription_mark_valid`, `subscription_mark_invalid`,
`offline_playback_allowed`) must stay reachable at
`crate::offline_mode::X` via `pub use ui_forward::*; pub use
subscription::*;` — `main.rs`/`playback.rs`/settings panels call these.

## Coupling / watch out
- `SUBSCRIPTION: Mutex<Option<SubscriptionStateStore>>` is shared across
  `subscription.rs` and `purge.rs` (both lock it) — keep it defined in
  `mod.rs` so both submodules can `use super::SUBSCRIPTION;`.
- `init_for_user` (subscription.rs) calls `spawn_subscription_purge_check()`
  (purge.rs) at the end — cross-file call, needs
  `use super::purge::spawn_subscription_purge_check;`.
- The purge consumer's comment explicitly calls out an init-order dependency
  (`offline::activate` before `offline_mode::init_for_user`) — do not
  reorder calls during the split, only reorganize file locations.
- `apply_status` is called from both `start_ui_forwarder`'s loop and
  indirectly relied on by `check_now`'s "checking" flag clear timing —
  keep it and `start_ui_forwarder` together in `ui_forward.rs`.

## Verify after split
- `cargo build -p qbz` (this crate has no `#[cfg(test)]` in this file).
- Smoke-test offline mode toggle in Settings, login/logout (subscription
  store bind/teardown), and the D2 recovery banner still function per the
  `run` skill if available.
