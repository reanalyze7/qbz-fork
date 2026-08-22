# crates/qbz/src/library_all.rs (433 lines)

## Summary
The "Library > All" mixed-feed controller (webplayer `/user-library/all`
equivalent): fans out to the existing per-type favorites/local loaders,
normalizes each into a `Feed` item, merges by a recency proxy, then applies
search/source-switch/genre/sort filtering entirely in Rust into
`LibraryAllState.items_visible` (Slint only renders the precomputed result).

## Proposed split
By pipeline stage — a `library_all/` module directory with a thin `mod.rs`
re-export; the fan-out loader further split by source since it's the
largest chunk (lines 61-265, ~205 lines on its own).

- `library_all/mod.rs` (~20 lines) — module declarations +
  `pub use` for `Feed`, `load_library_all`, `apply_library_all`, `set_sort`,
  `derive`, `artwork_jobs` so `crate::library_all::*` paths stay unchanged.
- `library_all/types.rs` (~40 lines) — the `Feed` struct (the plain `Send`
  data produced on the worker thread) and the small `rank(i, n)` helper.
- `library_all/loader/mod.rs` (~45 lines) — `load_library_all` orchestrator:
  calls each source helper below, appends into one `Vec<Feed>`, sorts by
  `added_rank`, returns `Ok(feed)`.
- `library_all/loader/favorites.rs` (~65 lines) — the "Favorites: tracks +
  albums" block (current lines 68-119): `load_fav_tracks_albums(runtime) ->
  Vec<Feed>`.
- `library_all/loader/following.rs` (~55 lines) — the "Following: artists +
  labels" block (lines 122-171): `load_following(runtime) -> Vec<Feed>`.
- `library_all/loader/playlists.rs` (~65 lines) — the "Playlists:
  owned/hearted = favorites, followed = following" block (lines 174-229):
  `load_playlists(runtime) -> Vec<Feed>`.
- `library_all/loader/local.rs` (~30 lines) — the "Local favorites" block
  (lines 233-256): `load_local() -> Vec<Feed>` (sync, no runtime needed).
- `library_all/apply.rs` (~45 lines) — `to_item` (Feed → `LibraryFeedItem`)
  and `apply_library_all` (push into `LibraryAllState`, call `derive`).
- `library_all/derive.rs` (~105 lines) — `set_sort` (PlaylistView-style
  toggle) and `derive` (search + source-switch + genre + sort filtering into
  `items_visible`).
- `library_all/artwork.rs` (~25 lines) — `artwork_jobs` (cover-download job
  builder over the current visible feed).

## Re-export surface
`library_all/mod.rs` is the public-API surface: re-export `Feed`,
`load_library_all`, `apply_library_all`, `set_sort`, `derive`,
`artwork_jobs` so every caller keeps using `crate::library_all::load_library_all(...)`
etc. unchanged.

## Coupling / watch out
- `Feed` has ~20 public-ish fields (some via `..Default::default()`); once
  split, every `loader/*.rs` file constructs `Feed { ... }` literals, so
  `types.rs`'s `Feed` fields must all be `pub(crate)` (or `pub`) — currently
  fine as private-within-module fields since it's all one file today.
- Each loader helper needs the same `Runtime` type alias
  (`type Runtime = Arc<AppRuntime<SlintAdapter>>`) — keep that alias in
  `library_all/mod.rs` or `types.rs` and import it into every loader file
  rather than redefining it four times.
- The `rank(i, n)` helper is called identically in all four loader blocks
  (`rank(i, n)` where `n` = that source's item count) — keep it in
  `types.rs` (or a `library_all/rank.rs` if preferred) and import it into
  each `loader/*.rs`.
- `load_library_all`'s final sort (`feed.sort_by(|a, b| a.added_rank.partial_cmp(...))`)
  must stay in the orchestrator (`loader/mod.rs`), AFTER all four sources
  have been appended — don't accidentally sort per-source.
- `derive`'s "if ALL three Qobuz switches are off, treat as no-filter"
  special case (lines 361-366) and the local-bypass logic (`is_local` skips
  the Qobuz switches entirely) are easy to lose track of when isolated in
  their own file — preserve the comment explaining why.
- `crate::local_favorites::list()` (used only in `loader/local.rs`) and
  `crate::genre_filter::selected_names("library-all")` (used only in
  `derive.rs`) are the only two other in-crate module dependencies beyond
  `favorites` — make sure each import lands in the right new file.

## Verify after split
- `cargo build -p qbz`
- `cargo test -p qbz` (no existing unit tests in this file; confirm nothing
  else regresses).
- Grep for `library_all::` usage across `qbz` to confirm import paths are
  unaffected (main window load, refresh triggers).
- Manual smoke test: open Library > All, confirm the merged feed order
  (favorites/following/playlists/local interleaved by recency), then
  exercise search, the purchases/favorites/following/local switches
  (including the "all off = show everything" fallback), genre filter, and
  each sort field (date/title/artist) in both directions.
