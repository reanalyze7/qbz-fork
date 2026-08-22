# crates/qbz/src/myqbz.rs (598 lines)

## Summary
My QBZ controller for the Mixtapes & Collections index grids: DB read/create
helpers over `qbz_mixtape::repo`, an offline-availability filter, string/
label helpers, the mosaic `MixtapeCardItem` builder, sort/search/filter
logic, apply/rebuild of the two grids' Slint models, and navigation
(load-then-render-then-spawn-artwork) glue.

## Proposed split
By responsibility, following the file's own `// ─── section ───` comment
banners (already a clean seam) — same flat-to-directory conversion as
`artwork.rs` (`myqbz.rs` → `myqbz/mod.rs` + siblings).

- `myqbz/mod.rs` (~40 lines) — module doc, imports, `Grid` enum (35-40), the
  two `LazyLock<Mutex<Vec<MixtapeCollection>>>` caches (29-33), and `pub use`
  re-exports of every public item in the submodules so `crate::myqbz::X`
  paths used from `main.rs`/`artwork.rs` (`crate::myqbz::set_mosaic_cover`)
  keep resolving unchanged.
- `myqbz/db.rs` (~55 lines) — "DB read path" section (44-97): `list_collections`,
  `create_collection`, `kind_from_str`. The I/O-over-`qbz_mixtape::repo` layer.
- `myqbz/offline.rs` (~95 lines) — "offline availability (D11.c)" section
  (99-191): `OfflineAvailability` struct + `item_available`,
  `offline_availability`, `retain_available_offline`.
- `myqbz/labels.rs` (~65 lines) — "string helpers" section (193-254):
  `kind_str`, `label_for`, `album_count_label`, `small_qobuz_url`,
  `cell_target`. Pure string/formatting logic, no Slint types.
- `myqbz/card.rs` (~105 lines) — "model builders" section (256-360):
  `card_item` (the `MixtapeCardItem` builder) and `set_mosaic_cover`. The one
  function that touches `slint::Image`/decoding directly.
- `myqbz/sort_filter.rs` (~35 lines) — "sort / filter" section (362-394):
  `sort_collections`, `passes_search`. Pure, no Slint dependency — the
  clearest pure-computation module in this file.
- `myqbz/render.rs` (~90 lines) — "apply / rebuild" section (396-502):
  `set_loading`, `apply`, `rebuild`, `set_sort`, `reset`. The Slint-model
  read/write layer; depends on `db.rs`'s caches (via `mod.rs`), `labels.rs`,
  `sort_filter.rs`, and `card.rs`.
- `myqbz/artwork_jobs.rs` (~30 lines) — "artwork jobs" section (504-535):
  `artwork_jobs`.
- `myqbz/navigate.rs` (~60 lines) — "navigation" section (537-598):
  `navigate`. Depends on everything else (db, offline filter, render,
  artwork_jobs).

## Re-export surface
`myqbz/mod.rs` is the public surface: `pub use db::*; pub use offline::*;
pub use card::{set_mosaic_cover}; pub use render::*; pub use
artwork_jobs::artwork_jobs; pub use navigate::navigate; pub use Grid;`. The
`crate::myqbz::set_mosaic_cover` call from `artwork.rs`'s apply match and
`crate::myqbz::navigate`/`Grid` calls from wherever the sidebar nav dispatches
into My QBZ must keep resolving with zero caller changes — `main.rs`'s `mod
myqbz;` line needs no edit.

## Coupling / watch out
- `MIXTAPES_CACHE`/`COLLECTIONS_CACHE` statics (currently top-of-file) are
  read by `render.rs` (`apply`/`rebuild`) and written by `render.rs::apply` —
  keep them defined ONCE in `mod.rs` (or a small `cache.rs`) with
  `pub(super)` visibility so both `render.rs` and any future module can reach
  them; do not accidentally duplicate the `LazyLock` statics.
- `card_item` (in `card.rs`) calls `crate::artwork::load_local_cover` directly
  — this is a real cross-module coupling with `artwork.rs`'s own in-flight
  split (see that plan); if both splits land in the same PR wave, verify
  `load_local_cover` keeps the same signature/re-export path.
- `Grid` enum (Mixtapes/Collections) is matched on in `render.rs`,
  `artwork_jobs.rs`, and `navigate.rs` — must be `pub(crate)` or `pub` visible
  from all three sibling files; define it once in `mod.rs`.
- `set_mosaic_cover` is called from OUTSIDE this module tree
  (`crate::artwork`'s `MyQbzMixtapeCover`/`MyQbzCollectionCover` apply arms)
  — must stay re-exported at `crate::myqbz::set_mosaic_cover`, not buried as
  `pub(crate)` inside `card.rs` only.
- `navigate`'s closure captures `weak: slint::Weak<AppWindow>`, `handle:
  tokio::runtime::Handle`, `image_cache: ImageCache` and calls
  `artwork::spawn_loads` — confirm the `ImageCache` type import path
  (`crate::artwork::{self, ArtworkJob, ArtworkTarget, ImageCache}`) is
  preserved in whichever file ends up defining/calling `navigate`.

## Verify after split
- `cargo build -p qbz` (main binary).
- `cargo clippy -p qbz`.
- `grep -rn "myqbz::" crates/qbz/src` — confirm every call site (sidebar nav
  dispatch, `artwork.rs`'s apply match) still resolves without an import-path
  change.
- Manual smoke test: open the My QBZ tab, switch between Mixtapes/Collections,
  change sort/search/kind-filter, confirm mosaic covers still populate and
  the offline-availability filter still hides fully-unavailable collections
  while offline.
