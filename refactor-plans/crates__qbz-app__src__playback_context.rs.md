# crates/qbz-app/src/playback_context.rs (321 lines)

## 1. Summary

Defines the "playback context" model — the semantic origin of playback
(album/playlist/radio/search/etc.): the `ContextType`/`ContentSource`
enums, the `PlaybackContext` struct with its navigation methods
(next/upcoming/advance), and a `ContextManager` singleton wrapper
(`Mutex<Option<PlaybackContext>>`) that commands use to read/mutate the
current context — plus a full test suite.

## 2. Proposed module split

| New file | Owns | ~lines |
|---|---|---|
| `playback_context/mod.rs` | Module decls + re-exports | ~15 |
| `playback_context/types.rs` | `ContextType`, `ContentSource` enums (pure data, serde derives) | ~30 |
| `playback_context/context.rs` | `PlaybackContext` struct + its inherent `impl` (`new`, `next_track_id`, `upcoming_track_ids`, `advance`, `has_next`, `total_tracks`, `display_info`) — pure logic, no locking | ~90 |
| `playback_context/manager.rs` | `ContextManager` struct + `impl Default` + `impl` (the `Mutex`-guarded read/write wrapper: `set_context`, `clear_context`, `get_context`, `has_context`, `next_track_id`, `upcoming_track_ids`, `advance_context`, `set_position`, `append_track_ids`) | ~95 |
| `playback_context/tests.rs` | The `#[cfg(test)] mod tests` block (7 tests) | ~105 |

This is a textbook pure-data / stateful-wrapper split: `types.rs` +
`context.rs` are pure value types and pure methods; `manager.rs` is the
only file holding the `Mutex`-guarded shared-state wrapper.

## 3. Re-export / public API surface

`playback_context/mod.rs` re-exports the same three public items:

```rust
mod context;
mod manager;
mod types;
#[cfg(test)]
mod tests;

pub use context::PlaybackContext;
pub use manager::ContextManager;
pub use types::{ContentSource, ContextType};
```

Every caller doing `use qbz_app::playback_context::{PlaybackContext,
ContextManager, ContextType, ContentSource};` keeps working unchanged.

## 4. Tricky coupling to watch out for

- `ContextManager`'s methods are thin `Mutex::lock().unwrap()` wrappers
  that call straight through to `PlaybackContext`'s own methods
  (`next_track_id`, `upcoming_track_ids`, `advance`) — `manager.rs` needs
  `use super::context::PlaybackContext;` and must NOT duplicate any of
  that logic; the split should be a pure move, not a rewrite.
- The doc comment atop the current file ("A playback context describes
  the semantic origin of playback... it is not the queue itself") is
  file-level context that matters for anyone reading `context.rs` in
  isolation — carry the relevant parts of it into `context.rs`'s module
  doc comment rather than losing it, since it explains a non-obvious
  design decision (context vs. queue).
- `ContextManager` is very likely constructed once as a long-lived
  `Arc<ContextManager>` or similar in `qbz-app`'s DI/state wiring —
  confirm via grep where `ContextManager::new()` is called before
  assuming the manager's lifecycle is unaffected by the file move (it
  isn't — pure module reorg — but worth confirming no macro/derive
  assumes the type's module path, e.g. no `#[serde(rename = "path::to::Type")]`
  anywhere).

## 5. What to verify after the real split

- `cargo build -p qbz-app` and `cargo test -p qbz-app playback_context::`
  — all 7 tests stay green
  (`playback_context_reports_next_and_upcoming_tracks`,
  `playback_context_advance_updates_position_until_end`,
  `playback_context_display_info_matches_existing_labels`,
  `context_manager_sets_clears_and_reports_context`,
  `context_manager_updates_position_by_track_id`,
  `context_manager_appends_radio_refill_track_ids`).
- Grep the workspace for `playback_context::` usages outside this crate
  (likely `qbzd` or `qbz-ui` playback commands) to confirm import paths
  still resolve.
- Smoke-test actual playback: start an album, hit next/prev enough times
  to exhaust the context, confirm "queue finished"-style behavior is
  unaffected (this logic backs the CLI's `render_advance` "queue
  finished" case in `qbzd`, so an end-to-end check touches both crates).
