# crates/qbz/src/discover_browse.rs (245 lines)

## Summary
Slint controller for the Discover "View all" full-list page: paginated
fetch (`fetch_pages`) with artist-blacklist filtering, `navigate` (initial
open + first page load + artwork fan-out), `load_more` (next-page fetch +
append), client-side search filtering (`apply_filter`), and an
artwork-job builder (`artwork_jobs`).

## Proposed split
By responsibility — data-fetch/pure-transform vs Slint-state-mutating UI
glue, matching the pure/IO/render convention:

- `discover_browse/mod.rs` (~20 lines) — module doc (lines 1-21 adapted) +
  `pub use` re-export of `navigate`, `load_more`, `apply_filter` (the three
  functions called from outside this module, e.g. from `main.rs`'s
  callback wiring and from other views that trigger `discover-view-all`).
- `discover_browse/fetch.rs` (~65 lines) — lines 32-71: `PAGE_SIZE` const +
  `fetch_pages` (the async network fetch + blacklist-filter + card mapping;
  no Slint types involved beyond the `AppRuntime`/`SlintAdapter` generic
  handle, so this is the closest thing to a "pure-ish I/O" module here).
- `discover_browse/navigate.rs` (~90 lines) — lines 73-137: `navigate` (the
  initial open + first-page load, including the `upgrade_in_event_loop`
  UI-thread callbacks and artwork spawn).
- `discover_browse/load_more.rs` (~90 lines) — lines 139-228: `load_more`,
  `append_albums`, `apply_filter` (pagination continuation + the
  model-append + the search-filter rebuild, which is called both from
  `append_albums`'s tail and externally on `search-changed`).
- `discover_browse/artwork.rs` (~15 lines) — lines 230-244: `artwork_jobs`
  helper, shared by both `navigate.rs` and `load_more.rs`.

## Re-export surface
`discover_browse/mod.rs` re-exports `navigate`, `load_more`, and
`apply_filter` (the three functions with external callers) via `pub use
navigate::navigate; pub use load_more::{load_more, apply_filter}; ` — the
existing callers doing `crate::discover_browse::navigate(...)`,
`crate::discover_browse::load_more(...)`, `crate::discover_browse::
apply_filter(...)` (likely from `main.rs`'s Slint callback registrations,
and possibly from wherever "search-changed" is wired) must keep resolving
unqualified.

## Coupling / watch out
- `fetch_pages` is called by BOTH `navigate` and `load_more` — keep it
  `pub(crate)` in `fetch.rs` and `use crate::discover_browse::fetch::
  fetch_pages;` (or a local `use super::fetch::fetch_pages;` if `mod.rs`
  declares `mod fetch; mod navigate; mod load_more;` as siblings) from both
  call sites.
- `artwork_jobs` is likewise called by both `navigate` and `load_more` —
  same cross-submodule `use` needed.
- `apply_filter` is called from THREE places: at the tail of `navigate`'s
  success branch (line 123), at the tail of `append_albums` (line 202,
  itself called from `load_more`'s success branch), AND presumably from an
  external `search-changed` callback registration elsewhere in the crate —
  keep it `pub fn` and re-exported at the top level as planned; do not
  make it `pub(crate)` only.
- `append_albums` and `apply_filter` both take `&AppWindow` and read/write
  `window.global::<DiscoverBrowseState>()` — if split into separate files
  they need identical `use crate::{AppWindow, DiscoverBrowseState}` (or
  equivalent) imports.
- `crate::home::map_album` / `crate::home::card_to_item` (the shared
  Discover-home mappers) are called from `fetch.rs` (`map_album`) and from
  `navigate.rs`/`load_more.rs` (`card_to_item`) — these live in a sibling
  `home` module untouched by this split; just make sure each new file's
  `use crate::home::{...}` names the right subset.
- `crate::artist_blacklist::{is_enabled, ids_snapshot, album_ids_snapshot}`
  and `qbz_core::core::discover_album_blacklisted` are only used inside
  `fetch_pages` — self-contained to `fetch.rs`.

## Verify after split
- `cargo check -p qbz` (this is a Slint-bin-internal module, not
  re-exported outside the `qbz` crate).
- No `#[cfg(test)]` module exists in this file today — verification is
  compile + manual/smoke-test only.
- Smoke-test in the running app: open Discover, click "View all" on any
  rail (e.g. New Releases), confirm the grid loads, scroll to trigger
  `load_more`, type into the search box to confirm `apply_filter` still
  narrows the grid without breaking artwork loading, and switch the
  genre filter to confirm `navigate` re-fires from offset 0.
