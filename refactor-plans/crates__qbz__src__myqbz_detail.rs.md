# crates/qbz/src/myqbz_detail.rs (1399 lines)

## 1. Summary
Read-mostly controller for the My QBZ "Collection/Mixtape DETAIL" view: loads
one `MixtapeCollection` from the local DB, precomputes every display string
and hero-mosaic, exposes filter/sort/search/select-mode toolbar logic, an
"expanded"-mode inline-track fetch+cache, an async `resolveItems` pass
(source/quality/type resolution with an offline-cache fallback), and
source-split artwork job dispatch — wired together by `navigate`.

## 2. Proposed module split
By domain/responsibility, mirroring the file's own `// ──── section ────`
banners (already a near-complete cut plan):

| New file | Owns | ~lines |
|---|---|---|
| `myqbz_detail/mod.rs` | Module decls, re-exports, the module doc comment, `GLOBAL_RUNTIME`/`set_runtime`/`global_runtime`, the three thread-locals (`FULL_ITEMS`, `INLINE_CACHE`, `PREFS_HYDRATED`, `RESOLVE_CACHE`) since almost every sibling module touches them | ~110 |
| `myqbz_detail/strings.rs` | Pure string/enum helpers: `kind_str`, `kind_label`, `play_mode_str`, `source_str`, `item_type_str`, `album_count_label`, `type_label`, `tracks_text`, `year_text`, `inline_cache_key`, `track_duration_str`, `inline_track_title` | ~110 |
| `myqbz_detail/model.rs` | `ResolvedItem` struct, `to_item` (row builder), `track_to_item` | ~130 |
| `myqbz_detail/toolbar.rs` | `persist_prefs`, `refresh_view`, `search`, `set_sort`, `set_type_filter`, `toggle_source_filter`, `reset_filters`, `set_view_mode`, `set_filter` (uses `strings.rs` + thread-locals) | ~130 |
| `myqbz_detail/selection.rs` | `toggle_select_mode`, `toggle_item_select`, `selected_positions`, `selected_full_items`, `clear_selection` | ~100 |
| `myqbz_detail/hero.rs` | `apply_hero_mosaic`, `set_hero_cover` | ~70 |
| `myqbz_detail/inline_tracks.rs` | `full_item_by_source_id`, `with_row_by_source_id`, `ensure_expanded` (expanded-mode fetch + cache) | ~130 |
| `myqbz_detail/resolve.rs` | `resolve_from_tracks`, `resolve_offline_cached`, `resolve_items` (the async resolveItems pass) | ~190 → split further into `resolve.rs` (dispatch/orchestration, `resolve_items`) and `resolve/offline.rs` (`resolve_offline_cached`) + `resolve/from_tracks.rs` (`resolve_from_tracks`) if still >130 |
| `myqbz_detail/artwork.rs` | `ArtworkJobSplit` struct, `artwork_jobs`, `dispatch_artwork`, `set_row_artwork` | ~100 |
| `myqbz_detail/lifecycle.rs` | `get_collection` (DB read), `reset`, `apply`, `apply_not_found`, `navigate` (top-level nav entry point) | ~180 → split `navigate` into its own `navigate.rs` if it stays large |

This keeps each file to one clear concern: string/formatting logic, the
render-row builder, toolbar state transitions, selection, hero mosaic,
inline-track expansion, the async resolver, artwork dispatch, and the
top-level open/apply/reset lifecycle.

## 3. Re-export / public API surface
`myqbz_detail/mod.rs` re-exports everything current external callers use
(`main.rs` and other view controllers call `myqbz_detail::navigate`,
`myqbz_detail::persist_prefs`, `myqbz_detail::source_str`,
`myqbz_detail::item_type_str`, etc. — grep shows `source_str`/`item_type_str`
are `pub fn` at module scope and likely reused by sibling controllers):

```rust
mod artwork;
mod hero;
mod inline_tracks;
mod lifecycle;
mod model;
mod resolve;
mod selection;
mod strings;
mod toolbar;

pub use artwork::{artwork_jobs, dispatch_artwork, set_row_artwork, ArtworkJobSplit};
pub use hero::set_hero_cover;
pub use inline_tracks::ensure_expanded;
pub use lifecycle::{apply, apply_not_found, get_collection, navigate, reset};
pub use resolve::resolve_items;
pub use selection::{clear_selection, selected_full_items, selected_positions,
    toggle_item_select, toggle_select_mode};
pub use strings::{item_type_str, source_str};
pub use toolbar::{persist_prefs, refresh_view, reset_filters, search,
    set_filter, set_sort, set_type_filter, set_view_mode, toggle_source_filter};
pub use {set_runtime, global_runtime}; // stay in mod.rs itself, or re-export from a `runtime.rs`
```

Every existing `crate::myqbz_detail::X` callsite in `main.rs` (and possibly
`myqbz_play.rs`) keeps working unchanged.

## 4. Tricky coupling / shared-state to watch out for
- The four thread-locals (`FULL_ITEMS`, `INLINE_CACHE`, `PREFS_HYDRATED`,
  `RESOLVE_CACHE`) are read/written from almost every proposed module
  (`toolbar.rs`, `model.rs`, `inline_tracks.rs`, `resolve.rs`,
  `lifecycle.rs`) — they must live in `mod.rs` (or a dedicated `state.rs`) and
  be `pub(super)` / `pub(crate)` so siblings can reach them; do NOT duplicate
  them per-file.
- `to_item` (in `model.rs`) reads BOTH `RESOLVE_CACHE` and `INLINE_CACHE` —
  it is the single place that re-hydrates a row after any toolbar re-derive,
  so it must not be split away from the caches it reads.
- `resolve_items`/`resolve_offline_cached` capture `crate::offline_mode`,
  `crate::myqbz_play::fetch_item_tracks`, and `crate::artwork` — these
  cross-module deps stay the same regardless of file location, just re-verify
  imports compile after the move.
- `navigate` (in `lifecycle.rs`) is the one function that ties `reset` →
  `apply` → `artwork_jobs`/`dispatch_artwork` → `resolve_items` together in a
  specific order across the async boundary (`upgrade_in_event_loop`); keep
  that orchestration in one function even if its helpers move to other files.
- `GLOBAL_RUNTIME`/`set_runtime`/`global_runtime` back the mutation-reload
  paths from OTHER files (cover upload/rename/delete) that re-run `navigate`
  — confirm no other crate file expects these symbols directly at
  `crate::myqbz_detail::set_runtime` (should still resolve via re-export).

## 5. What to verify after the real split
- `cargo build -p qbz` (the whole `qbz` binary crate, since this is UI glue
  with no dedicated test file visible here).
- Grep the workspace for `myqbz_detail::` to find every external caller
  (`main.rs` wiring, possibly cover-upload/rename modals) and confirm each
  import path still resolves.
- Smoke-test in the running app: open a Mixtape/Collection detail from the My
  QBZ grid, verify hero mosaic renders, toggle list/grid/expanded view modes,
  filter/sort/search, multi-select + bulk action, and confirm the async
  resolveItems quality/source badges still populate (online) and the
  offline-cache fallback still works (toggle offline mode).
