# crates/qbz/src/playlist.rs (1,085 lines)

## Summary
Playlist detail-view controller: fetches a Qobuz playlist, merges it with any
local sidecar rows (Seam A), and drives the shared `TrackItem` row model for
in-page search/sort/custom-order/drag-reorder, multi-select, bulk removal, and
custom artwork — the online-playlist twin of `mix.rs`/`local_playlist.rs`.

## Proposed split

- `playlist/mod.rs` (~40 lines) — module doc, `pub use` re-exports of every
  public item below so `crate::playlist::X` keeps resolving unchanged.
- `playlist/view_state.rs` (~140 lines) — the `FULL_ITEMS`/`QUERY`/`SORT`
  thread-locals, `custom_key`, `duration_secs`, `refresh_view`,
  `filter_tracks`, `set_sort`, `set_track_artwork`. This is the search/sort
  rendering core everything else calls into.
- `playlist/custom_order.rs` (~230 lines) — the `CUSTOM_ORDER` thread-local,
  `custom_seed_keys`, `full_item_ids`, `swap_full_items`, `move_full_item`,
  `load_or_init_custom`, `persist_custom`, `apply_custom_order`, `move_track`,
  `reorder_track`. Still borderline over 130 by itself; if so split further
  into `custom_order/io.rs` (load/persist, the two DB-facing functions) and
  `custom_order/reorder.rs` (move_track/reorder_track/swap/move, the pure
  reordering logic operating on the in-memory map).
- `playlist/load.rs` (~180 lines) — `PlaylistData`, `interleave_rows`, `load`
  (the async fetch + merge), `truncate_words`.
- `playlist/apply.rs` (~110 lines) — `reset`, `apply`, `apply_local_items`,
  `artwork_jobs`, `current_tracks`, `shuffled_tracks` — the "push loaded data
  into PlaylistState" half, kept separate from the fetch half in `load.rs`
  since `apply` is UI-thread-only while `load` is async/off-thread.
- `playlist/row_item.rs` (~90 lines) — `to_item`, `mmss` — the `Track` ->
  `TrackItem` mapper, pure enough to stand alone and likely reusable/shared
  logic worth isolating.
- `playlist/custom_artwork.rs` (~30 lines) — `set_custom_artwork`,
  `clear_custom_artwork`.
- `playlist/multi_select.rs` (~60 lines) — `recount_selected`,
  `set_multi_select`, `clear_selection`, `select_all`.
- `playlist/removal.rs` (~200 lines) — `SelectedRow`, `selected_rows`,
  `row_for_id`, `selected_queue_tracks`, `RemovalSplit`, `split_for_removal`
  (Seam D namespace-split removal + bulk queueing).

## Re-export surface
`playlist/mod.rs` re-exports everything at `crate::playlist::*` (functions,
`PlaylistData`, `SelectedRow`, `RemovalSplit`) so `main.rs` and other callers
that currently do `crate::playlist::apply(...)`, `crate::playlist::load(...)`,
etc. need zero changes.

## Tricky coupling / watch out
- `FULL_ITEMS`, `QUERY`, `SORT`, `CUSTOM_ORDER` are all UI-thread-only
  `thread_local!`s referenced across nearly every proposed file
  (`view_state.rs`, `custom_order.rs`, `removal.rs` via `row_for_id`). They
  must stay defined in exactly one file (`view_state.rs` for the first three,
  `custom_order.rs` for the last) with the others accessing them via
  `super::view_state::FULL_ITEMS` etc. — do not duplicate the `thread_local!`
  macro invocation.
- `CURRENT` (the Qobuz-only track cache) and `MIXED` (the sidecar-mixed flag)
  are `static`s read/written by `apply`/`reset` (in `apply.rs`) but also read
  by `current_tracks`/`shuffled_tracks` (also `apply.rs`, fine) and by
  `split_for_removal`/`selected_queue_tracks` (`removal.rs`) — keep `CURRENT`
  defined in `apply.rs` and have `removal.rs` reference it via
  `super::apply::CURRENT` (or promote to a shared `statics.rs` if that reads
  cleaner).
- `interleave_rows` is `pub(crate)` and its doc references the exact Tauri
  `displayTracks` slot-interleave contract (E1-E3 fix-forwards) — keep the
  doc comment attached when moving, it's load-bearing context for anyone
  touching it later.
- `custom_order.rs`'s `is_local` bool threading (Seam E: `(track_id, is_local)`
  keys) must stay consistent across `custom_seed_keys`, `load_or_init_custom`,
  `persist_custom`, and `custom_key` in `view_state.rs` — these two files are
  the tightest coupling in the split.
- `to_item` in `row_item.rs` calls `crate::artist_blacklist::stamp_row`,
  `crate::fav_cache::is_favorite`, `crate::offline_cache::is_cached`,
  `crate::quality::detail` — cross-module dependencies to preserve, not an
  issue for the split itself but worth noting for whoever reviews the diff.

## What to verify after the real split
- `cargo build -p qbz` and `cargo test -p qbz` (no `#[cfg(test)]` block exists
  in this file today, so no unit tests to keep green here specifically, but
  confirm the crate-wide test suite still passes).
- Grep for `crate::playlist::` across `crates/qbz/src/` (especially
  `main.rs`, `local_playlist.rs`, `playlist_suggestions.rs`) to confirm every
  call site still resolves through `playlist/mod.rs`'s re-exports.
- Manual smoke test via the `run` skill: open an online playlist, search/sort
  it, enter multi-select and bulk-remove a track, drag-reorder in custom sort,
  and open a mixed (online + local sidecar) playlist to exercise
  `interleave_rows`/`MIXED`.
