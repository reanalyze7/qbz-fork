# crates/qbz-app/src/settings/subscription.rs (299 lines)

## 1. Summary

Subscription-validity tracking for offline-download compliance: the
`SubscriptionState` DTO, `SubscriptionStateStore` (a SQLite-backed store
with grace-period logic — `mark_valid`/`mark_invalid`/
`should_purge_offline_cache`/`offline_playback_allowed`), the
`SubscriptionStateState` type alias + two constructor free functions, and
a test suite.

## 2. Proposed module split

| New file | Owns | ~lines |
|---|---|---|
| `subscription/mod.rs` | Module decls + re-exports; `GRACE_PERIOD_SECS` constant + its rationale doc comment | ~30 |
| `subscription/state.rs` | `SubscriptionState` struct + `impl Default` | ~25 |
| `subscription/store.rs` | `SubscriptionStateStore` (the SQLite `open_at`/`new`/`new_at`/`get_state`/`mark_valid`/`mark_invalid`/`mark_offline_cache_purged`/`should_purge_offline_cache`/`offline_playback_allowed`) — the I/O-bound persistence layer | ~140 |
| `subscription/handle.rs` | `SubscriptionStateState` type alias, `create_subscription_state`, `create_empty_subscription_state` — the shared-handle wiring used by app DI | ~15 |
| `subscription/tests.rs` | The entire `#[cfg(test)] mod tests` block (`unique_test_dir` helper + all 5 tests) | ~105 |

`store.rs` is the pure-I/O module (SQLite connection, all queries);
`state.rs` is pure data; `handle.rs` is the DI-facing `Arc<Mutex<...>>`
wrapper — a clean data/IO/wiring split.

## 3. Re-export / public API surface

`subscription/mod.rs`:

```rust
mod handle;
mod state;
mod store;
#[cfg(test)]
mod tests;

const GRACE_PERIOD_SECS: i64 = 30 * 24 * 60 * 60;

pub use handle::{create_empty_subscription_state, create_subscription_state, SubscriptionStateState};
pub use state::SubscriptionState;
pub use store::SubscriptionStateStore;
```

Every caller doing `use qbz_app::settings::subscription::{SubscriptionStateStore,
SubscriptionState, SubscriptionStateState, create_subscription_state};`
keeps working unchanged.

## 4. Tricky coupling/shared state to watch out for

- `GRACE_PERIOD_SECS` is used inside `store.rs` (`should_purge_offline_cache`,
  `offline_playback_allowed`) and referenced directly in `tests.rs`
  (`100 + GRACE_PERIOD_SECS - 1` etc.) — keep it visible to both, e.g.
  declare as `pub(super) const` in `mod.rs` and `use super::GRACE_PERIOD_SECS;`
  in both submodules, or duplicate the constant only if the "grace
  period" doc comment context needs to travel with the store logic
  (prefer the single shared constant to avoid drift).
- `SubscriptionStateStore::new()` calls `dirs::data_dir()` directly
  (global data dir) while `new_at()`/`open_at()` take an explicit path —
  this dual-entry-point pattern (production vs. test) must stay intact;
  the test suite exclusively uses `new_at()` with `unique_test_dir()`,
  so no test should accidentally start hitting the real global data dir.
- `offline_playback_allowed`'s doc comment explicitly calls out the "NO
  degraded 30-second-preview path" design decision (an explicit owner
  requirement) — this must move verbatim with the method into
  `store.rs`, not get summarized away, since it documents an
  intentional constraint a future editor must not "fix."
- `SubscriptionStateState = Arc<Mutex<Option<SubscriptionStateStore>>>`
  in `handle.rs` needs `use super::store::SubscriptionStateStore;`.

## 5. What to verify after the real split

- `cargo build -p qbz-app` and
  `cargo test -p qbz-app settings::subscription::` — all 5 tests green
  (default-valid-access, invalid-since preserves first observation,
  purge waits for grace period and runs once, offline-playback-allowed
  gate, mark_valid clears invalid_since).
- Grep the workspace for `subscription::SubscriptionStateStore`,
  `create_subscription_state`, `SubscriptionStateState` usages (likely
  the app's session-lifecycle / login-verdict wiring in `qbz-app` or the
  Slint host `qbz` crate) to confirm import paths still resolve.
- Smoke-test: sign in with a valid subscription, confirm
  `offline_playback_allowed` stays true; if feasible, simulate an
  invalid-subscription verdict and confirm the grace-period countdown
  and eventual offline-cache purge trigger identically to before the
  split (this touches the host-side session lifecycle mentioned in the
  file's module doc, so a full compliance end-to-end check spans beyond
  this one file).
