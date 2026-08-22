# crates/qbz/src/playlist_browse.rs (383 lines)

## Summary
Slint controller for the "Qobuz Playlists — View all" full-list page: fetches
`/discover/playlists` pages faceted by a category tag + shared genre selection,
maps them to Slint items, and drives pagination / client-side search filtering
/ tag selection / artwork loading for `PlaylistBrowseState`.

## Proposed split
By responsibility (data-fetch/mapping vs UI-thread mutation vs navigation
entry points) — this is UI-glue code (async fetch + `upgrade_in_event_loop`
mutation), so the split is "by concern" rather than strict pure/IO/render:

- `playlist_browse/mod.rs` (~40 lines) — module doc, imports, `PAGE_SIZE`
  const, the `SELECTED_TAG` static + `selected_tag()` helper (1-45), and
  re-exports (`pub use navigate::{navigate, load_more, select_tag}; pub use
  filter::apply_filter;`).
- `playlist_browse/model.rs` (~50 lines) — `BrowseCard` struct, `map_browse`,
  `to_item` (50-83): the pure data-shaping layer (Qobuz DTO -> Slint item).
- `playlist_browse/fetch.rs` (~20 lines) — `fetch_page` (88-102): the one
  async network call, isolated so it's easy to see/reason about the API
  contract independent of UI-thread plumbing.
- `playlist_browse/navigate.rs` (~140 lines) — `navigate()` (110-201): the
  main "open this page" entry point (reset state, fetch tags+page concurrently,
  populate the view, spawn artwork). This is the biggest single function; if it
  runs over 130 alone, consider extracting the artwork/error-handling tail
  (166-200) into a small `finish_navigate_page(...)` helper in the same file
  rather than a new file (splitting mid-function across files hurts
  readability more than it helps here).
- `playlist_browse/load_more.rs` (~50 lines) — `load_more()` (206-247) +
  `append_playlists()` (251-263): pagination continuation, kept together since
  `load_more` is the only caller of `append_playlists`.
- `playlist_browse/filter.rs` (~25 lines) — `apply_filter()` (269-288): the
  client-side search-query filter, small and self-contained, reused by
  `navigate`/`select_tag`/`load_more` indirectly.
- `playlist_browse/select_tag.rs` (~75 lines) — `select_tag()` (295-366): tag
  switching (radio-flag update + re-fetch page 0).
- `playlist_browse/artwork.rs` (~15 lines) — `artwork_jobs()` (371-383): tiny
  pure helper building `ArtworkJob`s from a page of cards.

## Re-export surface
`playlist_browse/mod.rs` re-exports `navigate`, `load_more`, `select_tag`, and
`apply_filter` — the four functions called from outside this module (wired to
Slint callbacks / other navigation controllers, e.g. from the Home rail's
"View all" link and `PlaylistBrowseActions`). `fetch_page`, `map_browse`,
`to_item`, `append_playlists`, `artwork_jobs`, `BrowseCard`, `selected_tag()`
stay private (`pub(super)` where cross-file access is needed within the
module, e.g. `fetch.rs`'s `fetch_page` is called from both `navigate.rs` and
`load_more.rs` and `select_tag.rs`).

## Coupling / watch out
- `SELECTED_TAG` is a module-level `static Mutex<String>` — process-global
  state living outside the Slint global on purpose (per the module doc: "so
  the fetch tasks can read it off the UI thread"). It's read/written from
  `navigate`, `load_more` (via `selected_tag()`), and `select_tag` (which
  writes it directly). Keep this static in `mod.rs` (or a tiny
  `playlist_browse/state.rs`) with `pub(super)` visibility so every submodule
  file can reach it — do NOT duplicate the static per-file.
- `fetch_page`, `map_browse`, `to_item`, `append_playlists`, `apply_filter` are
  all called from multiple entry points (`navigate`, `load_more`,
  `select_tag`) — the cross-file `pub(super)` visibility on ALL of these is
  the main thing to get right; a build will fail loudly if missed, but grep for
  each function name across the new files first to enumerate every call site
  before splitting.
- No `#[cfg(test)]` block exists in this file today — nothing to relocate, but
  per the project's testing rule this is an opportunity to note (not fix here)
  that `apply_filter`'s substring-match logic and `map_browse`'s subtitle
  formatting are the two easiest candidates for a future unit test pass since
  they're pure functions once split out.

## Verify after split
- `cargo build -p qbz` (this crate holds the Slint app).
- `cargo build --workspace` since `qbz-app::shell::AppRuntime` and the
  `qbz-slint`-side Slint-generated types (`AppWindow`, `PlaylistBrowseState`,
  etc.) are referenced.
- Manual/smoke check per the `run` skill: open the app, navigate to Qobuz
  Playlists "View all", confirm tag switching, scroll-triggered load-more, and
  the search box filter all still work — this file has no automated tests
  today so a real click-through is the only verification available.
