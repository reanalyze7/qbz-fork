# crates/qbz/src/nav.rs (423 lines)

## 1. Summary

The shell's browser-like navigation history: the `NavEntry` enum (every
navigable page/tab in the app, ~25 variants, `Serialize`/`Deserialize`
for "resume where you left off"), the internal `Entry`/`History` structs
with a `thread_local` stack + scroll-position bookkeeping, and the public
API (`record`, `push_or_replace_search`, `reset_root`, `go_back`,
`go_forward`, `can_back`, `can_forward`, `set_live_scroll`, `current`)
plus a test suite.

## 2. Proposed module split

| New file | Owns | ~lines |
|---|---|---|
| `nav/mod.rs` | Module decls + re-exports; brief module-level doc | ~15 |
| `nav/entry.rs` | The `NavEntry` enum (all ~25 variants + their doc comments) — pure data, no logic | ~125 |
| `nav/history.rs` | `Entry`, `History` structs, the `thread_local!` statics (`HISTORY`, `LIVE_SCROLL`), `set_live_scroll`, `current`, `live_scroll` | ~50 |
| `nav/navigation.rs` | `push_or_replace_search`, `record`, `reset_root`, `go_back`, `go_forward`, `can_back`, `can_forward` — the stateful stack-mutation API | ~130 |
| `nav/tests.rs` | The entire `#[cfg(test)] mod tests` block | ~105 |

By-domain split: `entry.rs` is pure data (the enum), `history.rs` is the
storage primitives + thread-local statics, `navigation.rs` is the
mutating operations built on top of them.

## 3. Re-export / public API surface

`nav/mod.rs`:

```rust
mod entry;
mod history;
mod navigation;
#[cfg(test)]
mod tests;

pub use entry::NavEntry;
pub use history::{current, set_live_scroll};
pub use navigation::{
    can_back, can_forward, go_back, go_forward, push_or_replace_search, record, reset_root,
};
```

Every caller doing `use qbz::nav::{NavEntry, record, go_back, ...};`
(the shell's navigation buttons, the ~30 `record` call sites across
views, the "startup page" persistence code that serializes `NavEntry`)
keeps working unchanged.

## 4. Tricky coupling/shared/state to watch out for

- `HISTORY` and `LIVE_SCROLL` are both `thread_local!` statics declared
  in the SAME block today; `navigation.rs`'s functions
  (`record`/`go_back`/`go_forward`/`reset_root`) directly touch
  `HISTORY.with(...)` AND call `live_scroll()`/`set_live_scroll(...)`
  from `history.rs` — after the split these need
  `use super::history::{HISTORY, live_scroll, set_live_scroll};` with
  `HISTORY` and `Entry`/`History` visible as `pub(super)` (not fully
  private) so `navigation.rs` can reach them.
- `Entry`/`History` structs must stay `pub(super)` or `pub(crate)`
  (currently private to the file) — check nothing outside `nav.rs`
  reaches into them directly (unlikely, since only `NavEntry` and the
  free functions are the intended public surface), but confirm via grep.
- `NavEntry`'s `PartialEq` derive is load-bearing: `record()`'s dedup
  check (`h.entries.get(h.cursor).map(|e| &e.nav) == Some(&entry)`)
  depends on it — keep the derive list (`Clone, Debug, PartialEq,
  Serialize, Deserialize`) intact when moving the enum to `entry.rs`.
- The scroll-restore design note in the top-of-file doc comment
  (recording live scroll on every navigation, `NavState.restore-scope`
  handoff to the Slint side) is important cross-cutting context — keep
  the full doc comment on `nav/mod.rs` (module-level) rather than
  scattering pieces of it, since it explains WHY `live_scroll`/
  `set_live_scroll` exist at all, which isn't obvious from `history.rs`
  alone.
- `NavEntry`'s serde is used for "Startup page = where you left off"
  persistence (`ui_prefs.last_nav`) — confirm no `#[serde(rename =
  "nav::NavEntry")]`-style path-dependent attribute exists anywhere
  (a quick grep of the enum's derive line and any manual serde impl
  elsewhere) before assuming the move is serialization-format-neutral;
  a plain derive round-trips fine on a pure module reorg since it
  serializes variant names, not paths.

## 5. What to verify after the real split

- `cargo build -p qbz` and `cargo test -p qbz nav::` — all 6 tests green
  (`record_then_back_and_forward`, `record_truncates_forward_history`,
  `search_entry_round_trips_history`,
  `payload_free_and_id_carrying_round_trip_history`,
  `record_dedupes_current_entry`,
  `scroll_is_stamped_on_leave_and_restored_on_return`).
- Grep the workspace for `nav::NavEntry`, `nav::record`, `nav::go_back`,
  `nav::go_forward` usages across `qbz-app`/`qbz-ui`/Slint callbacks to
  confirm all ~30+ call sites still resolve.
- Smoke-test: navigate through several pages (Discover tabs, an album,
  search, back to Home), use the back/forward buttons, confirm scroll
  position is restored on both directions, and confirm the "resume on
  launch" behavior still restores the last `NavEntry` after a restart.
