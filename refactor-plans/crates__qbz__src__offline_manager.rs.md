# crates/qbz/src/offline_manager.rs (436 lines)

## Summary
Offline Cache Manager controller: loads the artist→album→track rollup +
size stats from the offline index.db into `OfflineManagerState`, applies
toolbar filters (artist rail / sort / show-only-failed), builds album cover
thumbnails, and handles multi-select + the size-limit edit. Per-item cache
actions are delegated to `offline_cache.rs` (the other file in this batch).

## Proposed split
By responsibility — filters/formatting are pure, the rollup build is the
big data-transform function, multi-select is UI-thread model editing:

- `offline_manager/mod.rs` (~20 lines) — module doc, `pub use` re-exports of
  `rebuild`, `load`, `select_artist`, `set_sort`, `toggle_failed`,
  `toggle_select`, `set_all_selected`, `selected_track_ids`, `set_limit`,
  plus `pub(crate) use format::human_size;` (used elsewhere in the crate per
  its `pub(crate)` visibility today).
- `offline_manager/filters.rs` (~45 lines) — the `Filters` struct, `FILTERS`
  static, `filters()`, `current_filters()`. Self-contained toolbar state.
- `offline_manager/format.rs` (~35 lines) — `human_size` (keep `pub(crate)`
  — verify no other file in the crate already imports
  `crate::offline_manager::human_size` before narrowing it further),
  `track_status_int`, `fmt_duration`, `album_size`, `cover_path`,
  `COVER_DECODE_SIZE`. Pure formatting/derivation helpers.
- `offline_manager/rowdata.rs` (~15 lines) — the `RowData` struct (the
  worker-built, `Send` intermediate row).
- `offline_manager/rebuild.rs` (~200 lines) — `rebuild` itself: the big
  async function that reads the DB, builds the artist→album→track rollup,
  applies filters/sort, builds `RowData`, and pushes `OfflineManagerState`
  on the UI thread. This is the heart of the file and the hardest to shrink
  further without fragmenting one coherent data pipeline — if it's still
  over 130 lines after extracting the `weak.upgrade_in_event_loop` UI-push
  tail into its own `push_state` helper, do that split too:
  - `offline_manager/rebuild.rs` (~140 lines) — DB read → rollup → filter →
    sort → `Vec<RowData>`.
  - `offline_manager/push.rs` (~60 lines) — the `upgrade_in_event_loop`
    closure that converts `RowData` → `OfflineRow` and sets every
    `OfflineManagerState` property.
- `offline_manager/load.rs` (~10 lines) — `load` (mark loading + spawn
  rebuild).
- `offline_manager/toolbar.rs` (~25 lines) — `select_artist`, `set_sort`,
  `toggle_failed`.
- `offline_manager/selection.rs` (~90 lines) — `recount`, `toggle_select`,
  `set_all_selected`, `selected_track_ids`. The multi-select in-place model
  editing (shift-range logic via `crate::selection`).
- `offline_manager/limit.rs` (~15 lines) — `set_limit`.

## Re-export surface
`offline_manager/mod.rs` is the `mod offline_manager;` target. Every
currently-public fn must stay reachable at `crate::offline_manager::X`
(most already are `pub`; `human_size` is `pub(crate)` — preserve that
narrower visibility, don't widen it during the split).

## Coupling / watch out
- `rebuild` is called from THIS batch's `offline_cache.rs` (multiple call
  sites: `remove_album`, `redownload_track`, `redownload_album`, `clear_all`,
  `remove_cached_inner`) as `crate::offline_manager::rebuild(weak.clone())`
  — a cross-file dependency already noted in that file's plan. Keep
  `rebuild` at the same public path after the split.
- `Filters` (filters.rs) is read by `rebuild.rs` (`current_filters()`) and
  written by `toolbar.rs` (`select_artist`/`set_sort`/`toggle_failed`) — a
  shared-state dependency across three new files; make sure all three can
  `use super::filters::current_filters;` / `filters()`.
- `human_size` in `format.rs` is `pub(crate)`, meaning something ELSE in the
  `qbz` crate already imports it directly — grep for
  `offline_manager::human_size` before finalizing the split so the actual
  split doesn't accidentally make it private.
- `cover_path`/`COVER_DECODE_SIZE` (format.rs) are used only inside
  `rebuild.rs`'s per-album cover resolution — fine to keep `pub(super)` or
  private-to-`offline_manager`, no external callers expected.
- `recount` (selection.rs) reads `st.get_rows()` via `VecModel` downcast —
  the same pattern `toggle_select`/`set_all_selected` use; keep all three
  together since they share the "iterate the model, only touch `kind ==
  \"track\"` rows" invariant.

## What to verify after the real split
- `cargo build -p qbz`.
- `cargo test -p qbz` (no dedicated tests in this file; ensure crate suite
  still green).
- Manual smoke: open the Offline Cache Manager, filter by artist, sort by
  size, toggle "show only failed", multi-select tracks (plain click, shift
  range, select-all), and edit the size limit slider — confirm the rollup
  and toolbar state stay consistent across the split module boundaries.
