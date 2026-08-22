# crates/qbz/src/myqbz_play.rs (760 lines)

## Summary
My QBZ (Collection/Mixtape) detail-view playback: hero Play/Shuffle,
per-row play/play-next/add-to-queue, bulk enqueue for multi-select, inline
per-track actions inside an expanded item, and headless item-boundary
skip-to-next/previous helpers (Part B). Wires `qbz-mixtape`'s
`ProdItemResolver` to the shared queue.

## Proposed split
The file already has explicit `// ─── Part B ───` and section-comment
boundaries — split along those plus by CTA type:

- `myqbz_play/mod.rs` (~60 lines) — `Runtime` type alias, `RowMode` enum +
  `parse`, `resolve_local` closure fn, `pub use` of submodules.
- `myqbz_play/resolve.rs` (~90 lines) — `resolve_collection`,
  `resolve_single_item`, `fetch_item_tracks` (the three Qobuz-client-
  snapshotting resolver wrappers).
- `myqbz_play/hero.rs` (~130 lines) — `play_all_tracks`, `play_all`,
  `shuffle`, `persist_album_shuffle`, `touch_play` (whole-collection replace
  paths + the shuffle side-effect persistence).
- `myqbz_play/row_actions.rs` (~90 lines) — `play_item`, `item_action`
  (per-row play/play-next/add-to-queue).
- `myqbz_play/bulk.rs` (~90 lines) — `bulk_enqueue`,
  `resolve_bulk_qobuz_track_ids` (multi-select bulk operations).
- `myqbz_play/inline_track.rs` (~90 lines) — `InlineTrackMode` enum +
  `parse`, `play_inline_track` (expanded-view single-track actions).
- `myqbz_play/load.rs` (~15 lines) — `load_collection`.
- `myqbz_play/skip.rs` (~80 lines) — Part B: `skip_to_next_item`,
  `skip_to_previous_item` (headless item-boundary nav, currently
  `#[allow(dead_code)]` pending UI wiring — keep both fns AND their
  `#[allow(dead_code)]` attributes + the long module-level comment block
  explaining why there's no UI trigger yet).

## Re-export surface
`myqbz_play/mod.rs` stays the `mod myqbz_play;` target. Public fns called
from `main.rs`/Slint callbacks (`play_all`, `shuffle`, `play_item`,
`item_action`, `bulk_enqueue`, `play_inline_track`,
`resolve_bulk_qobuz_track_ids`, `skip_to_next_item`,
`skip_to_previous_item`) re-exported via `pub use hero::*; pub use
row_actions::*; pub use bulk::*; pub use inline_track::*; pub use skip::*;`.
`pub(crate)` fns (`resolve_collection`, `play_all_tracks`,
`fetch_item_tracks`, `load_collection`) stay `pub(crate)` and reachable at
the same `crate::myqbz_play::X` paths for `main.rs`/`myqbz_detail.rs`.

## Coupling / watch out
- `resolve_local` (mod.rs) is used by EVERY resolver-building call site
  (`resolve.rs`, `bulk.rs` build their own `ProdItemResolver::new(&client,
  resolve_local)`) — keep it as a free `pub(super)` fn in `mod.rs` so every
  submodule can `use super::resolve_local;`.
- `RowMode`/`InlineTrackMode` are separate near-identical enums (Play/
  PlayNext/AddToQueue vs Play/PlayNext/PlayLater) — do not merge them during
  a mechanical split even though they look similar; they serve different
  action-string vocabularies (`row_actions.rs` vs `inline_track.rs`).
  `RowMode` stays defined in `mod.rs` (used across the file); consider
  moving `InlineTrackMode` into `inline_track.rs` since it's local to that
  file's single caller.
- `hero.rs`'s `shuffle` calls `persist_album_shuffle` which reaches into
  `crate::myqbz_detail::navigate` to reload the open detail — cross-module
  coupling already present, just preserve the `crate::` path.
- Every Qobuz-resolver call site repeats the same "snapshot client under
  `RwLock` read, clone it, bail with a toast if None" ~10-line pattern
  (`resolve_collection`, `resolve_single_item`, `fetch_item_tracks`,
  `bulk_enqueue`, `resolve_bulk_qobuz_track_ids`) — flag as an extraction
  opportunity (a `fn snapshot_client(runtime) -> Option<QobuzClient>`
  helper) for the REAL split PR, but don't do it silently as part of a
  mechanical line-count split (it changes error-message/log-line call
  sites).
- The Part B skip helpers explicitly document they must NEVER be wired into
  the normal transport (`playback::next()/previous()`) — preserve that
  comment verbatim wherever `skip.rs` ends up so a future contributor
  doesn't "helpfully" wire them in without reading the context.

## Verify after split
- `cargo build -p qbz` (no `#[cfg(test)]` in this file — flag as a gap).
- Smoke-test: hero Play, hero Shuffle (+ verify play_mode persists +
  overflow label flips if collection is open), per-row Play/Play
  Next/Add to Queue, bulk-select Play Next/Add to Queue, and the expanded
  inline-track menu actions — all six entry points changed files.
