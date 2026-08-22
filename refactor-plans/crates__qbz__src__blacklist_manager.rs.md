# crates/qbz/src/blacklist_manager.rs (352 lines)

## Summary
Blacklist Manager controller: loads the per-user artist blacklist, blocked
albums, and reco-dismissal ("Not interested") lists into `BlacklistState`,
applies search-as-you-type filtering controller-side, and runs the
toggle/remove/clear mutations for all three axes (artists, albums,
recommendations dismissals).

## Proposed split
By the three axes the file itself documents (artist blacklist / album
blacklist / reco-dismissal), plus a shared query/push core:

- `blacklist_manager/mod.rs` (~25 lines) — module doc, `pub use`
  re-exports of `set_image_cache`, `load`, `search_changed`,
  `toggle_enabled`, `remove`, `clear_all`, `set_tab`, `block_album`,
  `remove_album`, `clear_all_albums`, `remove_dismissed`.
- `blacklist_manager/query.rs` (~40 lines) — the `IMAGE_CACHE` static,
  `set_image_cache`, the `QUERY` static, `current_query`, `set_query`,
  `format_added`. Shared plumbing every axis's `build_*_items` uses.
- `blacklist_manager/artists.rs` (~90 lines) — `build_items`,
  `toggle_enabled`, `remove`, `clear_all` (the artist-axis actions).
- `blacklist_manager/albums.rs` (~110 lines) — `build_album_items`,
  `set_tab`, `block_album`, `remove_album`, `clear_all_albums` (the
  album-axis actions, including the `ArtworkJob` cover-load wiring).
- `blacklist_manager/dismissed.rs` (~45 lines) — `build_dismissed_items`,
  `remove_dismissed` (the reco-dismissal axis).
- `blacklist_manager/push.rs` (~40 lines) — `push`, `load`. The function
  that calls all three `build_*_items` and writes every `BlacklistState`
  field in one shot, plus the `load` entry point that wraps it with the
  loading flag.

## Re-export surface
`blacklist_manager/mod.rs` is the `mod blacklist_manager;` target. Every
currently-public fn (`set_image_cache`, `load`, `search_changed`,
`toggle_enabled`, `remove`, `clear_all`, `set_tab`, `block_album`,
`remove_album`, `clear_all_albums`, `remove_dismissed`) stays reachable at
`crate::blacklist_manager::X` via `pub use query::set_image_cache; pub use
push::load; pub use artists::*; pub use albums::*; pub use dismissed::*;`
(`search_changed` can live in `push.rs` next to `push`, since it's just
`set_query` + `push`).

## Coupling / watch out
- `push` (push.rs) calls all three `build_*_items` functions AND reads
  `IMAGE_CACHE` (query.rs) to kick off cover-load jobs from
  `build_album_items`'s returned `Vec<ArtworkJob>` — this is the one
  function that touches every other submodule; keep its import list
  complete (`artists::build_items`, `albums::build_album_items`,
  `dismissed::build_dismissed_items`, `query::IMAGE_CACHE`).
- `QUERY` (query.rs) is read by all three `build_*` functions via
  `current_query()` and written only by `search_changed` — every
  `build_*_items` in the three axis files needs `use
  super::query::current_query;`.
- Every mutation fn (`remove`, `clear_all`, `toggle_enabled`, `block_album`,
  `remove_album`, `clear_all_albums`, `remove_dismissed`) ends by calling
  `push(w)` to re-render — an intra-crate but cross-file (after split)
  dependency on `push.rs`; each axis file needs `use super::push::push;`.
- `block_album`/`remove_album` (albums.rs) also reach into
  `w.global::<crate::AlbumState>()` to flip `is_album_blocked` when the
  currently-open album is the one being blocked/unblocked — a coupling to
  the Album detail page's own state global, unrelated to the blacklist
  manager's own `BlacklistState`; keep this side-effect inside
  `block_album`/`remove_album` rather than trying to generalize it.
- `set_tab` is trivial (one-line global setter) but logically belongs with
  the album axis only because it's currently defined right before
  `block_album` — it actually controls all three tabs (0/1/2), so it may
  read better living in `push.rs` next to `load`/`push` instead. Either
  placement works; just re-export it either way.

## What to verify after the real split
- `cargo build -p qbz`.
- `cargo test -p qbz` (no dedicated tests in this file; ensure crate suite
  still green).
- Manual smoke: open Blacklist Manager, search-filter each of the three
  tabs, toggle blacklist enabled, remove one artist/one album/one dismissal,
  clear-all each axis, and block/unblock an album while its detail page is
  open (to exercise the `AlbumState` cross-talk).
