# crates/qbz/src/queue.rs (1198 lines)

## Summary
`QueueController` — the Queue sidebar's full controller: view state
(tab/search/page), the big `refresh_async` (pulls queue snapshot, builds
now-playing/upcoming/history/coverflow rows, gates coverflow rebuild by a
sequence-hash, dispatches artwork jobs), and every queue callback (play/
remove/reorder/favorite/infinite-play/save-as-playlist/etc).

## Proposed split
By concern — this is the single largest file in the whole slice; split
along its own section comments (`--- NOW PLAYING ---`, `--- Callbacks
---`, etc.) plus the free-standing artwork helpers at the bottom:

- `queue/mod.rs` (~110 lines) — `PAGE_SIZE`, `ViewState`, `QueueController`
  struct + its manual `Clone` impl, `RowData` struct, `fmt_duration`,
  `display_title`, `PageBounds`, `paginate`, `row_from`, `new()`,
  accessors (`runtime`/`weak`/`handle`), `pub use` of submodules.
- `queue/refresh.rs` (~330 lines) — `refresh`, `refresh_with_favorites`,
  `refresh_async` (the big one). Still way over 130 — split further by the
  numbered sections already in the code:
  - `queue/refresh/snapshot.rs` (~90 lines) — pulling `get_queue_state_full`,
    now-playing + filtered/paginated upcoming + history row-building.
  - `queue/refresh/coverflow.rs` (~110 lines) — building the flat coverflow
    model, the sequence-hash computation, and the seq-changed gate.
  - `queue/refresh/apply.rs` (~130 lines) — the `upgrade_in_event_loop`
    closure that pushes everything onto `QueueState` (prior-artwork
    snapshotting + reuse, model sets, coverflow windowed decode dispatch).
  - `queue/refresh/mod.rs` (~30 lines) — `refresh`/`refresh_with_favorites`
    entry points, calling into the three above in sequence.
- `queue/nav.rs` (~120 lines) — `play_upcoming`, `play_coverflow_upcoming`,
  `remove_upcoming`, `remove_all_after`, `reorder`, `current_page_len`,
  `resolve_upcoming_index` (index resolution + reordering callbacks).
- `queue/actions.rs` (~130 lines) — `play_history`, `clear`,
  `toggle_favorite`, `toggle_infinite_play`, `toggle_stop_after`,
  `is_infinite_play` (misc per-queue actions). Still near budget — could
  split `toggle_favorite`+`toggle_infinite_play`+`toggle_stop_after`
  (~80 lines) from `play_history`+`clear`+`is_infinite_play` (~50 lines).
- `queue/playlist.rs` (~55 lines) — `save_as_playlist`, `add_to_playlist`.
- `queue/paging_ui.rs` (~40 lines) — `prev_page`, `next_page`, `set_tab`,
  `search_changed` (trivial view-state setters + refresh trigger).
- `queue/artwork.rs` (~120 lines) — `ArtTarget` enum, `to_item_reuse`,
  `load_artwork`, `apply_queue_art` (the free-standing artwork pipeline
  glue, not `QueueController` methods).
- `queue/tests.rs` (~105 lines) — existing `#[cfg(test)] mod tests`.

## Re-export surface
`queue/mod.rs` stays the `mod queue;` target with `QueueController`,
`PAGE_SIZE` defined there. Every other file adds `impl QueueController`
blocks (`use super::QueueController;`) or free fns (`load_artwork`,
`apply_queue_art`, `ArtTarget`) that stay privately scoped to the module
tree (only `main.rs`/other controllers call `QueueController` methods, not
these helpers directly) — no public API surface changes for callers.

## Coupling / watch out
- **This is the highest-risk split in the whole gap-fill batch.** The
  `refresh_async` method threads a huge amount of local state (rows,
  coverflow hash, art job lists) through one function body; splitting it
  into `snapshot.rs`/`coverflow.rs`/`apply.rs` requires designing an
  intermediate struct to carry that state across the split boundaries —
  this is NOT a copy-paste split, flag it clearly for whoever does the real
  work.
- The **coverflow seq-hash gate is the single most load-bearing invariant**
  in the file (explained at length in comments): on a pure advance/jump the
  flat model must NOT be replaced (`set_coverflow_tracks` skipped), only
  `coverflow_index` moves — this is what avoids Repeater rebuilds/re-decodes.
  Any split must keep `seq_changed` computed BEFORE the event-loop closure
  and gate `set_coverflow_tracks`/`set_coverflow_tracks_rev` on it exactly
  as today.
- The **prior-artwork snapshot map** (`prior_all: HashMap<SharedString,
  Image>`) is built by reading every existing model (now-playing, upcoming,
  history, coverflow) INSIDE the event-loop closure, before any model is
  replaced — this "global map across lists" is explicitly why moved covers
  (queue -> now-playing -> history) don't re-decode; keep this construction
  intact if `apply.rs` is split from `coverflow.rs`.
- `last_coverflow_seq: Arc<Mutex<Option<u64>>>` on the controller struct is
  read+written inside `refresh_async` — must stay accessible from whichever
  file computes `seq_changed`.
- `resolve_upcoming_index` (nav.rs) is called by `play_upcoming`,
  `remove_upcoming`, `remove_all_after`, `reorder`, AND `add_to_playlist` —
  keep it `pub(super)` or similar so `playlist.rs`/`actions.rs` can call it
  via `use super::nav::...` if split apart, or just keep all its callers in
  `nav.rs` and have other files call through `self.resolve_upcoming_index`
  (works fine as an inherent method regardless of file).
- Tests reference `PAGE_SIZE`, `paginate`, `fmt_duration`, `display_title`,
  `row_from` — all defined in `mod.rs` per this plan, so `queue/tests.rs`
  needs `use super::*;` only, no further wiring.

## Verify after split
- `cargo test -p qbz queue::` — all 8 existing tests green
  (`fmt_duration_pads_seconds`, `display_title_appends_version`,
  `row_from_marks_playing_and_explicit`, and 5 `paginate_*` tests).
- `cargo build -p qbz` end to end.
- Manual/smoke test (critical given the complexity): play/pause/skip via
  queue, reorder drag, search filter, page navigation, toggle favorite on
  now-playing, and — most importantly — watch CPU/repaint behavior on
  simple track-advance vs. a real queue mutation (shuffle/add/remove) to
  confirm the coverflow rebuild-vs-index-only gate still behaves
  identically pre/post split (this is the one behavior a bad split could
  silently regress without any compile error).
