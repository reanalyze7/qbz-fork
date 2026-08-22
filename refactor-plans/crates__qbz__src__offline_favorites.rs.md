# crates/qbz/src/offline_favorites.rs (246 lines)

## 1. Summary

Builds the "playable favorites" rail shown under the OfflinePlaceholder
on the Favorites view while OFFLINE: `load()` async-computes the
intersection of favorited track ids with (offline-cache-ready ∪
library-download-copy) tracks from three local stores, populates a
Slint model; `play()` replaces the player queue with the rail starting
at a clicked row. Includes a small `RowData` struct, a static
`RAIL_QUEUE`, and pure helpers (`khz`, `index_queue_track`).

## 2. Proposed module split

| New file | Owns | ~lines |
|---|---|---|
| `offline_favorites/mod.rs` | Module decls + re-exports; the file's module-level doc comment (the three-source rail design, the intersection rule) | ~30 |
| `offline_favorites/model.rs` | `RowData` struct, `RAIL_QUEUE` static, `COVER_DECODE_SIZE` constant, pure helpers `khz`, `index_queue_track` | ~55 |
| `offline_favorites/load.rs` | The `load()` async function — reads the three local stores, builds `rows`/`queue`, logs the skipped-count diagnostic, pushes into the Slint model | ~130 |
| `offline_favorites/play.rs` | The `play()` function — reads `RAIL_QUEUE`, resolves the start index, replaces the player queue | ~30 |

By-domain split rather than strict pure/IO: `model.rs` holds the pure
value types/helpers and the shared static; `load.rs` is the one
I/O-heavy async builder; `play.rs` is the small consumer of the built
queue.

## 3. Re-export / public API surface

`offline_favorites/mod.rs`:

```rust
mod load;
mod model;
mod play;

pub use load::load;
pub use play::play;
```

`RowData`, `RAIL_QUEUE`, `khz`, `index_queue_track`, `COVER_DECODE_SIZE`
stay private to the module (`pub(super)` in `model.rs` so `load.rs` can
reach `RAIL_QUEUE`/`index_queue_track`/`khz`, and `play.rs` can reach
`RAIL_QUEUE`). Every external caller doing
`crate::offline_favorites::{load, play}` (the Favorites Slint view's
callbacks) keeps working unchanged.

## 4. Tricky coupling/shared state to watch out for

- `RAIL_QUEUE` (`static ... LazyLock<Mutex<Vec<QueueTrack>>>`) is
  written by `load()` and read by `play()` — this is the one genuinely
  shared piece of state crossing the new `load.rs`/`play.rs` boundary;
  it must live in `model.rs` (or `mod.rs`) as `pub(super)` so both
  submodules see the SAME static (not two independent ones — a classic
  copy-paste risk when splitting files that share a `static`).
  Double-check after the split that there's exactly one `RAIL_QUEUE`
  definition, not one accidentally duplicated into each file.
- `load()`'s comment about "ids with no local metadata are skipped
  (count logged)" ties directly to the diagnostic logging near the end
  of the function (`skipped` computation) — keep that logic and its
  log line together in `load.rs`, don't split the row-building from the
  diagnostic count.
- `RowData.cover: Option<crate::artwork::DecodedPixels>` — the
  worker-thread/UI-thread split (decode on worker, `slint::Image` build
  on UI thread via `upgrade_in_event_loop`) is itself already a
  pure/IO-style pattern *within* `load()`; don't further split `load()`
  in a way that separates the worker-thread block from the
  `upgrade_in_event_loop` closure, since they share `rows`/`queue` by
  move.
- `index_queue_track` embeds the "offline-copy" `QueueTrack` shape
  contract (`source = "qobuz_download"`, `is_local` semantics) that
  mirrors `crate::playback::local_queue_track`'s offline-copy arm per
  its doc comment — this cross-reference comment must travel with
  `index_queue_track` into `model.rs`.

## 5. What to verify after the real split

- `cargo build -p qbz` (no `#[cfg(test)]` module exists in this file
  today, so no unit tests to run — confirm this is still true after the
  split, i.e. no tests were silently dropped).
- Grep the workspace for `offline_favorites::load`/`offline_favorites::play`
  usages (the Favorites Slint view's `init`/row-click callbacks) to
  confirm import paths still resolve.
- Smoke-test: go offline (or simulate the OFFLINE state), open
  Favorites, confirm the playable-favorites rail populates with correct
  covers/titles/artists, click a row, confirm playback starts at that
  row and continues through the rest of the rail via the offline cache
  tier.
