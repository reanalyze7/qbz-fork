# crates/qbz/src/playlist_suggestions.rs (673 lines)

## Summary
The playlist "Suggested Songs" controller (T8): holds a process-global `Session`
(pool/pagination/exclude-sets) behind a `Mutex`, derives adaptive seed artists
from the open playlist's tracks, fetches/merges suggestion pages via
`qbz_reco`/`AppRuntime::core()`, filters/paginates/projects them onto
`PlaylistSuggestionsState`, and exposes the UI actions (activate, refresh,
add-track, dismiss-track, play-track, reset).

## Proposed split
By responsibility — state/session ↔ pure algorithm (adaptive artists, filtering)
↔ networking/fetch orchestration ↔ UI actions:

- `playlist_suggestions/mod.rs` (~50 lines) — module doc, constants
  (`VISIBLE_COUNT`, `INITIAL_POOL`, `EXPANDED_POOL`, `MAX_POOL`,
  `MIN_AVAILABLE_THRESHOLD`), type aliases (`Runtime`, `Handle`, `Weak`),
  `mod` declarations, `pub use` re-exports of `activate`, `refresh`, `add_track`,
  `play_track`, `dismiss_track`, `reset`.
- `playlist_suggestions/session.rs` (~90 lines) — `Phase` enum, `Session`
  struct, the `static SESSION: LazyLock<Mutex<Session>>`, and small session
  helpers if any are extracted (kept minimal since session is read/written
  directly elsewhere today).
- `playlist_suggestions/adaptive_artists.rs` (~130 lines) — `normalize`,
  `make_key`, `mmss`, `splitmix64`, `shuffle`, `extract_adaptive_artists` (all
  pure, no I/O — the deterministic seed-artist selection algorithm ported from
  Svelte).
- `playlist_suggestions/filter_project.rs` (~110 lines) — `filtered_indices`,
  `to_row`, `project` (pure-ish filtering + the UI-thread projection onto
  `PlaylistSuggestionsState`; kept together since `project` is what every other
  file calls after mutating `SESSION`).
- `playlist_suggestions/fetch.rs` (~150 lines) — `spawn_fetch`,
  `maybe_auto_expand`, `reload_open_playlist` (the async fetch/merge/error
  handling and the pool-exhaustion auto-refresh).
- `playlist_suggestions/actions.rs` (~180 lines) — `activate`, `refresh`,
  `add_track`, `play_track`, `dismiss_track`, `set_row_flag`, `reset` (the public
  UI-triggered entry points).

## Re-export surface
`playlist_suggestions/mod.rs` re-exports `activate`, `refresh`, `add_track`,
`play_track`, `dismiss_track`, `reset` at `crate::playlist_suggestions::*` (these
are the only symbols this module's callers use, based on the file's own public
API) so `main.rs`/wiring code needs zero import changes.

## Coupling / watch out
- `SESSION` (the `Mutex<Session>`) is read/written from nearly every file
  (`filter_project.rs`, `fetch.rs`, `actions.rs`) — it must live in one place
  (`session.rs`) and be referenced as `super::session::SESSION` (or re-exported
  privately via `pub(super) use session::SESSION;` in `mod.rs`) from the others.
  This is the file's central piece of shared mutable state — get the visibility
  right first before splitting further.
- `project()` (in `filter_project.rs`) is called from `fetch.rs` and
  `actions.rs` after every mutation — make sure it's `pub(super)` or `pub(crate)`
  so cross-submodule calls compile.
- `spawn_fetch`/`maybe_auto_expand` call each other recursively
  (`maybe_auto_expand` → `spawn_fetch` → on success → `maybe_auto_expand`) — keep
  them in the same file (`fetch.rs`) to avoid a circular-import headache.
- `extract_adaptive_artists` and `filtered_indices` are pure and easily unit
  testable in isolation once split out — a good opportunity (not required by
  this task) for future test coverage.
- Depends on `crate::playlist_suggestions_dismiss` (T10 store),
  `crate::playlist::current_tracks/load/apply/artwork_jobs`, `crate::playback`,
  `crate::artwork` — these cross-module calls are unaffected by an internal
  split as long as `use crate::...` lines are carried into whichever new file
  calls them.

## Verify after split
- `cargo check -p qbz` / `cargo build`.
- No existing unit tests in this file — none to keep green, but consider it low
  risk to add a couple for `extract_adaptive_artists`/`filtered_indices` while
  the file is open (optional, not required by the 130-line rule).
- Smoke-test: open a playlist with the "Suggested Songs" wand, confirm pool
  loads, paging/refresh/wrap-cycle works, add + dismiss update the list, and
  reset behaves correctly on playlist navigation.
